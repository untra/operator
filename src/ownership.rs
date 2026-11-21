use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, OnceLock, Weak};

use anyhow::{Context, Result};

use crate::config::Config;

const STATE_LOCK: &str = ".operator-owner.lock";
const REGISTRY_LOCK_SUFFIX: &str = ".owner.lock";

#[derive(Debug)]
pub struct ResourceGuard {
    file: Option<File>,
    path: PathBuf,
    released: Arc<(Mutex<bool>, Condvar)>,
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        if let Some(file) = &self.file {
            let _ = file.unlock();
        }
        drop(self.file.take());
        let (released, ready) = &*self.released;
        *released
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
        ready.notify_all();
    }
}

impl ResourceGuard {
    pub fn state(path: &Path) -> Result<Self> {
        let path = canonical_directory(path)?;
        Self::acquire(path.join(STATE_LOCK))
    }

    pub fn registry(path: &Path) -> Result<Self> {
        let path = canonical_file(path)?;
        let mut name = path.as_os_str().to_owned();
        name.push(REGISTRY_LOCK_SUFFIX);
        Self::acquire(PathBuf::from(name))
    }

    fn acquire(path: PathBuf) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .with_context(|| format!("Opening ownership lock {}", path.display()))?;
        match file.try_lock() {
            Ok(()) => Ok(Self { file: Some(file), path, released: Arc::new((Mutex::new(false), Condvar::new())) }),
            Err(TryLockError::WouldBlock) => anyhow::bail!(
                "Resource is already owned: {}. Stop its owning runtime before using this local command.",
                path.display()
            ),
            Err(TryLockError::Error(error)) => Err(error)
                .with_context(|| format!("Locking resource {}", path.display())),
        }
    }
}

pub fn canonical_directory(path: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(path)
        .with_context(|| format!("Creating ownership directory {}", path.display()))?;
    path.canonicalize()
        .with_context(|| format!("Resolving ownership directory {}", path.display()))
}

pub fn canonical_file(path: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(path)?;
    if absolute.exists() {
        return absolute.canonicalize().context("Resolving ownership file");
    }
    let parent = absolute.parent().context("Ownership file needs a parent")?;
    Ok(canonical_directory(parent)?.join(absolute.file_name().context("Missing file name")?))
}

fn share(
    guard_path: PathBuf,
    acquire: impl FnOnce() -> Result<ResourceGuard>,
) -> Result<Arc<ResourceGuard>> {
    type CachedGuard = (Weak<ResourceGuard>, Arc<(Mutex<bool>, Condvar)>);
    static GUARDS: OnceLock<Mutex<BTreeMap<PathBuf, CachedGuard>>> = OnceLock::new();
    let mut guards = GUARDS
        .get_or_init(Mutex::default)
        .lock()
        .map_err(|_| anyhow::anyhow!("Ownership guard table poisoned"))?;
    if let Some((weak, release)) = guards.get(&guard_path) {
        if let Some(guard) = weak.upgrade() {
            return Ok(guard);
        }
        let (released, ready) = &**release;
        drop(
            ready
                .wait_while(
                    released
                        .lock()
                        .map_err(|_| anyhow::anyhow!("Ownership release poisoned"))?,
                    |released| !*released,
                )
                .map_err(|_| anyhow::anyhow!("Ownership release poisoned"))?,
        );
    }
    let guard = Arc::new(acquire()?);
    debug_assert_eq!(guard.path, guard_path);
    guards.insert(
        guard_path,
        (Arc::downgrade(&guard), Arc::clone(&guard.released)),
    );
    Ok(guard)
}

pub fn state_guard(path: &Path) -> Result<Arc<ResourceGuard>> {
    let path = canonical_directory(path)?;
    share(path.join(STATE_LOCK), || ResourceGuard::state(&path))
}

pub fn registry_guard(path: &Path) -> Result<Arc<ResourceGuard>> {
    let path = canonical_file(path)?;
    let mut lock = path.as_os_str().to_owned();
    lock.push(REGISTRY_LOCK_SUFFIX);
    share(PathBuf::from(lock), || ResourceGuard::registry(&path))
}

#[derive(Debug)]
pub struct Ownership {
    state_path: PathBuf,
    auth_path: PathBuf,
    registry: PathBuf,
    _registry_guard: Arc<ResourceGuard>,
    states: BTreeMap<PathBuf, Arc<ResourceGuard>>,
}

impl Ownership {
    pub fn acquire(config: &Config) -> Result<Arc<Self>> {
        let registry = canonical_file(
            &config
                .profile_registry
                .clone()
                .unwrap_or_else(|| config.state_path().join(crate::profiles::REGISTRY_FILE)),
        )?;
        let registry_guard = registry_guard(&registry)?;
        let state_path = canonical_directory(&config.state_path())?;
        let auth_path = canonical_directory(&config.auth_state_path())?;
        let mut paths = BTreeSet::from([state_path.clone(), auth_path.clone()]);
        for loaded in crate::profiles::registered_configs_read_only(&registry)? {
            paths.insert(canonical_directory(&loaded.state_path())?);
            paths.insert(canonical_directory(&loaded.auth_state_path())?);
        }
        if let Some(path) = crate::profiles::auth_path_read_only(&registry)? {
            paths.insert(canonical_directory(&path)?);
        }
        let states = paths
            .into_iter()
            .map(|path| state_guard(&path).map(|guard| (path, guard)))
            .collect::<Result<_>>()?;
        Ok(Arc::new(Self {
            state_path,
            auth_path,
            registry,
            _registry_guard: registry_guard,
            states,
        }))
    }

    pub fn validate(&self, config: &Config) -> Result<()> {
        let registry = config
            .profile_registry
            .clone()
            .unwrap_or_else(|| config.state_path().join(crate::profiles::REGISTRY_FILE));
        anyhow::ensure!(
            canonical_file(&registry)? == self.registry,
            "Changing the profile registry requires restarting Operator"
        );
        anyhow::ensure!(
            canonical_directory(&config.state_path())? == self.state_path
                && canonical_directory(&config.auth_state_path())? == self.auth_path,
            "Changing ownership paths requires restarting Operator"
        );
        debug_assert!(self.states.contains_key(&self.state_path));
        Ok(())
    }
}
