use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::config::Config;
use crate::rest::state::ApiState;

pub const MAX_PROFILE_NAME_LENGTH: usize = 64;
pub const LEGACY_PROFILE_NAME: &str = "legacy";
const REGISTRY_FILE: &str = "profiles.sqlite";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS, ToSchema)]
#[ts(export)]
pub struct ProfileIdentity {
    pub id: Uuid,
    pub name: String,
}

impl Default for ProfileIdentity {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            name: LEGACY_PROFILE_NAME.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS, ToSchema)]
#[ts(export)]
pub struct ProfileSummary {
    pub id: Uuid,
    pub name: String,
    pub initialized: bool,
    pub is_default: bool,
}

pub fn validate_name(name: &str) -> Result<()> {
    anyhow::ensure!(
        !name.is_empty() && name.len() <= MAX_PROFILE_NAME_LENGTH
            && name.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-' || c == b'_'),
        "Configuration names must contain 1-{MAX_PROFILE_NAME_LENGTH} lowercase letters, digits, - or _"
    );
    Ok(())
}

pub fn registry_path() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .context("User configuration directory unavailable")?
        .join("operator")
        .join(REGISTRY_FILE))
}

fn open_registry(path: &Path) -> Result<Connection> {
    std::fs::create_dir_all(path.parent().context("Registry needs a parent directory")?)?;
    let connection = Connection::open(path)?;
    connection.busy_timeout(std::time::Duration::from_secs(5))?;
    connection.execute_batch("CREATE TABLE IF NOT EXISTS profiles (id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE, config_path TEXT NOT NULL UNIQUE); CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);")?;
    Ok(connection)
}

fn absolute_paths(config: &mut Config, root: &Path) {
    for value in [
        &mut config.paths.tickets,
        &mut config.paths.state,
        &mut config.paths.projects,
        &mut config.paths.worktrees,
    ] {
        let path = Path::new(value);
        if path.is_relative() {
            *value = root.join(path).to_string_lossy().into_owned();
        }
    }
}

pub fn load_registered(path: &Path, registry: &Path) -> Result<Config> {
    let path = canonical_config_path(path)?;
    let input = std::fs::read_to_string(&path)?;
    let defaults = serde_json::to_string(&Config::default())?;
    let mut config: Config = config::Config::builder()
        .add_source(config::File::from_str(&defaults, config::FileFormat::Json))
        .add_source(config::File::from_str(&input, config::FileFormat::Toml))
        .build()?
        .try_deserialize()?;
    config.config_file = Some(path.clone());
    config.profile_registry = Some(registry.to_path_buf());
    let connection = open_registry(registry)?;
    let (id, name): (String, String) = connection
        .query_row(
            "SELECT id, name FROM profiles WHERE config_path = ?1",
            [path.to_string_lossy().as_ref()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .context("Configuration is not registered")?;
    config.profile = ProfileIdentity {
        id: Uuid::parse_str(&id)?,
        name,
    };
    config.server_auth_path = connection
        .query_row(
            "SELECT value FROM settings WHERE key='auth_path'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .map(PathBuf::from);
    absolute_paths(
        &mut config,
        path.parent().context("Configuration needs a parent")?,
    );
    crate::config::validate_targets(&config)?;
    Ok(config)
}

pub fn select(name: &str) -> Result<Config> {
    validate_name(name)?;
    let registry = registry_path()?;
    let connection = open_registry(&registry)?;
    let path: String = connection
        .query_row(
            "SELECT config_path FROM profiles WHERE name = ?1",
            [name],
            |row| row.get(0),
        )
        .with_context(|| format!("Unknown configuration '{name}'"))?;
    load_registered(Path::new(&path), &registry)
}

pub fn rename_registered(config: &mut Config, name: &str) -> Result<()> {
    validate_name(name)?;
    let registry = config
        .profile_registry
        .as_deref()
        .context("Configuration is not registered")?;
    let connection = open_registry(registry)?;
    let changed = connection
        .execute(
            "UPDATE profiles SET name = ?1 WHERE id = ?2",
            params![name, config.profile.id.to_string()],
        )
        // Names are UNIQUE in the registry; a collision is a user-facing
        // conflict, not a database error to be shown verbatim.
        .map_err(|error| match error {
            rusqlite::Error::SqliteFailure(code, _)
                if code.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                anyhow::anyhow!("Configuration name '{name}' already exists")
            }
            other => anyhow::Error::new(other),
        })?;
    anyhow::ensure!(changed == 1, "Configuration is not registered");
    config.profile.name = name.to_owned();
    Ok(())
}

/// The registry key for a configuration file.
///
/// Canonicalising the file only once it exists would key the first run and
/// every later one differently (on macOS `/var` resolves to `/private/var`),
/// registering a second row for the same workspace and minting a new id - which
/// silently invalidates its licence. The parent directory always exists, so
/// canonicalise that and rejoin the file name.
fn canonical_config_path(path: &Path) -> Result<PathBuf> {
    let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
        return Ok(path.to_path_buf());
    };
    std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    Ok(parent.canonicalize()?.join(name))
}

pub fn register_legacy(config: &mut Config) -> Result<()> {
    register_in(config, &registry_path()?)
}

/// Register `config` in the registry at `registry`, adopting its identity if a
/// row already exists for this config path. Split from [`register_legacy`] so
/// tests can point at a temporary registry instead of the user's own.
fn register_in(config: &mut Config, registry: &Path) -> Result<()> {
    let registry = registry.to_path_buf();
    let mut connection = open_registry(&registry)?;
    // Immediate, not deferred: every `operator` invocation runs this, so two
    // concurrent processes would otherwise both read "no row", both pick the
    // same name and one would lose the INSERT to the UNIQUE constraint and
    // exit. Taking the write lock up front makes them queue on `busy_timeout`.
    let transaction =
        connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    absolute_paths(config, &std::env::current_dir()?);
    let path = canonical_config_path(&config.operator_config_path_for())?;
    config.config_file = Some(path.clone());
    config.profile_registry = Some(registry);
    let existing = transaction.query_row(
        "SELECT id, name FROM profiles WHERE config_path = ?1",
        [path.to_string_lossy().as_ref()],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    );
    let mut registered = false;
    match existing {
        Ok((id, name)) => {
            config.profile = ProfileIdentity {
                id: Uuid::parse_str(&id)?,
                name,
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // Always mint: `profile.id` round-trips through config.toml, and the
            // licence pins it, so an id arriving from the file is an assertion
            // this process is not entitled to trust.
            config.profile.id = Uuid::new_v4();
            let mut suffix = 1;
            let base = config.profile.name.clone();
            validate_name(&base)?;
            while transaction
                .query_row(
                    "SELECT 1 FROM profiles WHERE name = ?1",
                    [&config.profile.name],
                    |_| Ok(()),
                )
                .is_ok()
            {
                suffix += 1;
                config.profile.name = format!("{base}-{suffix}");
                validate_name(&config.profile.name)?;
            }
            registered = true;
            transaction.execute(
                "INSERT INTO profiles (id,name,config_path) VALUES (?1,?2,?3)",
                params![
                    config.profile.id.to_string(),
                    config.profile.name,
                    path.to_string_lossy()
                ],
            )?;
            transaction.execute(
                "INSERT OR IGNORE INTO settings (key,value) VALUES ('default_profile',?1)",
                [config.profile.id.to_string()],
            )?;
        }
        Err(error) => return Err(error.into()),
    }
    transaction.commit()?;
    if registered {
        config.save()?;
    }
    connection.execute(
        "INSERT OR IGNORE INTO settings (key,value) VALUES ('auth_path',?1)",
        [config.state_path().to_string_lossy().as_ref()],
    )?;
    config.server_auth_path = Some(PathBuf::from(connection.query_row(
        "SELECT value FROM settings WHERE key='auth_path'",
        [],
        |row| row.get::<_, String>(0),
    )?));
    if config.tickets_path().join("queue").is_dir()
        && !crate::startup::workspace_initialized(config)
    {
        crate::startup::mark_workspace_initialized(config)?;
    }
    Ok(())
}

pub struct ServerProfiles {
    registry: Mutex<Connection>,
    registry_path: PathBuf,
    states: RwLock<std::collections::HashMap<Uuid, ApiState>>,
    primary: ApiState,
    default_id: Uuid,
}

impl ServerProfiles {
    pub fn open(primary: ApiState) -> Result<Arc<Self>> {
        let config = primary.config();
        // Always on disk. An in-memory registry would accept a configuration,
        // report success, and lose it on restart - worse than refusing.
        let registry_path = config
            .profile_registry
            .clone()
            .unwrap_or_else(|| config.state_path().join(REGISTRY_FILE));
        let registry = open_registry(&registry_path)?;
        let mut default_id = registry
            .query_row(
                "SELECT value FROM settings WHERE key='default_profile'",
                [],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .and_then(|value| Uuid::parse_str(&value).ok())
            .unwrap_or(config.profile.id);
        let mut states = std::collections::HashMap::from([(config.profile.id, primary.clone())]);
        {
            let mut query = registry.prepare("SELECT config_path FROM profiles")?;
            for row in query.query_map([], |row| row.get::<_, String>(0))? {
                let path = row?;
                let loaded = match load_registered(Path::new(&path), &registry_path) {
                    Ok(loaded) => loaded,
                    Err(error) => {
                        tracing::warn!(config_path = %path, %error, "Registered configuration is unavailable");
                        continue;
                    }
                };
                if loaded.profile.id != config.profile.id {
                    let state = ApiState::with_auth(
                        loaded.clone(),
                        loaded.tickets_path(),
                        Arc::clone(&primary.auth),
                    );
                    states.insert(loaded.profile.id, state);
                }
            }
        }
        if !states.contains_key(&default_id) {
            default_id = config.profile.id;
            registry.execute(
                "INSERT INTO settings (key,value) VALUES ('default_profile',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                [default_id.to_string()],
            )?;
        }
        Ok(Arc::new(Self {
            registry: Mutex::new(registry),
            registry_path,
            states: RwLock::new(states),
            primary,
            default_id,
        }))
    }

    pub fn default_state(&self) -> ApiState {
        self.state(self.default_id)
            .unwrap_or_else(|| self.primary.clone())
    }

    pub fn state(&self, id: Uuid) -> Option<ApiState> {
        self.states
            .read()
            .expect("profile registry poisoned")
            .get(&id)
            .cloned()
    }

    pub fn list(&self) -> Vec<ProfileSummary> {
        let mut profiles: Vec<_> = self
            .states
            .read()
            .expect("profile registry poisoned")
            .values()
            .map(|state| self.summary_for(&state.config()))
            .collect();
        profiles.sort_by(|a, b| a.name.cmp(&b.name));
        profiles
    }

    pub fn summary(&self, config: &Config) -> ProfileSummary {
        self.summary_for(config)
    }

    fn summary_for(&self, config: &Config) -> ProfileSummary {
        ProfileSummary {
            id: config.profile.id,
            name: config.profile.name.clone(),
            initialized: crate::startup::workspace_initialized(config),
            is_default: config.profile.id == self.default_id,
        }
    }

    pub fn create(&self, name: &str) -> Result<ProfileSummary> {
        validate_name(name)?;
        let registry = self.registry.lock().expect("profile registry poisoned");
        anyhow::ensure!(
            !self.list().iter().any(|p| p.name == name),
            "Configuration name already exists"
        );
        let id = Uuid::new_v4();
        let root = self
            .registry_path
            .parent()
            .context("Missing registry directory")?
            .join("profiles")
            .join(id.to_string());
        let mut config = Config::default();
        config.profile = ProfileIdentity {
            id,
            name: name.into(),
        };
        config.config_file = Some(canonical_config_path(&root.join("config.toml"))?);
        config.profile_registry = Some(self.registry_path.clone());
        config.server_auth_path = Some(self.primary.config().auth_state_path());
        config.paths.tickets = root.join("tickets").to_string_lossy().into_owned();
        config.paths.state = root.join("state").to_string_lossy().into_owned();
        config.paths.worktrees = root.join("worktrees").to_string_lossy().into_owned();
        config.paths.projects = self
            .primary
            .config()
            .projects_path()
            .to_string_lossy()
            .into_owned();
        config.rest_api = self.primary.config().rest_api.clone();
        config.sessions.tmux.socket_name = format!("operator-{id}");
        config.save()?;
        registry.execute(
            "INSERT INTO profiles (id,name,config_path) VALUES (?1,?2,?3)",
            params![
                id.to_string(),
                name,
                config.operator_config_path_for().to_string_lossy()
            ],
        )?;
        let summary = self.summary_for(&config);
        let state = ApiState::with_auth(
            config.clone(),
            config.tickets_path(),
            Arc::clone(&self.primary.auth),
        );
        self.states
            .write()
            .expect("profile registry poisoned")
            .insert(id, state);
        Ok(summary)
    }

    pub async fn rename(
        &self,
        id: Uuid,
        name: String,
    ) -> Result<ProfileSummary, crate::rest::error::ApiError> {
        use crate::rest::error::ApiError;
        validate_name(&name).map_err(|e| ApiError::ValidationError(e.to_string()))?;
        let state = self
            .state(id)
            .ok_or_else(|| ApiError::NotFound("Configuration not found".into()))?;
        state
            .mutate_config(|config| {
                let mut registry = self.registry.lock().expect("profile registry poisoned");
                let transaction = registry
                    .transaction()
                    .map_err(|e| ApiError::InternalError(e.to_string()))?;
                if self.list().iter().any(|p| p.id != id && p.name == name) {
                    return Err(ApiError::Conflict(
                        "Configuration name already exists".into(),
                    ));
                }
                transaction
                    .execute(
                        "UPDATE profiles SET name=?1 WHERE id=?2",
                        params![name, id.to_string()],
                    )
                    .map_err(|e| ApiError::Conflict(e.to_string()))?;
                config.profile.name.clone_from(&name);
                // Commit before mutate_config saves: a registry row whose
                // config.toml lags is read back correctly next run; the reverse
                // loses the rename.
                transaction
                    .commit()
                    .map_err(|e| ApiError::InternalError(e.to_string()))?;
                Ok(self.summary_for(config))
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_reject_paths_unicode_and_empty_values() {
        for value in ["", "../escape", "Upper", "space name", "é", ".", "a/b"] {
            assert!(validate_name(value).is_err(), "{value}");
        }
        assert!(validate_name("demo_123-test").is_ok());
        assert!(validate_name(&"a".repeat(MAX_PROFILE_NAME_LENGTH + 1)).is_err());
    }

    /// The licence pins `profile_id`, and `config.toml` is user-editable, so
    /// the registry - not the file - decides which configuration this is.
    /// Otherwise a leaked licence plus a one-line edit defeats the binding.
    #[test]
    fn a_config_declared_id_cannot_override_the_registry() {
        let temp = tempfile::tempdir().unwrap();
        let registry = temp.path().join(REGISTRY_FILE);
        let root = temp.path().join("workspace");
        std::fs::create_dir_all(&root).unwrap();

        let mut config = Config::default();
        config.paths.state = root.to_string_lossy().into_owned();
        config.paths.tickets = root.join("tickets").to_string_lossy().into_owned();
        register_in(&mut config, &registry).unwrap();
        let assigned = config.profile.id;
        assert!(!assigned.is_nil());

        // Someone edits profile.id in config.toml and restarts.
        let mut tampered = Config::default();
        tampered.paths.state = root.to_string_lossy().into_owned();
        tampered.paths.tickets = root.join("tickets").to_string_lossy().into_owned();
        tampered.profile.id = Uuid::new_v4();
        register_in(&mut tampered, &registry).unwrap();

        assert_eq!(
            tampered.profile.id, assigned,
            "the registry's id must win over the one declared in config.toml"
        );
    }

    /// Renaming onto a name already in use is something a user can do by
    /// accident, so it must read as a conflict rather than as a SQLite error.
    #[test]
    fn renaming_onto_an_existing_name_reports_a_conflict() {
        let temp = tempfile::tempdir().unwrap();
        let registry = temp.path().join(REGISTRY_FILE);

        let mut configs = Vec::new();
        for i in 0..2 {
            let root = temp.path().join(format!("workspace-{i}"));
            std::fs::create_dir_all(&root).unwrap();
            let mut config = Config::default();
            config.paths.state = root.to_string_lossy().into_owned();
            config.paths.tickets = root.join("tickets").to_string_lossy().into_owned();
            register_in(&mut config, &registry).unwrap();
            configs.push(config);
        }

        let taken = configs[0].profile.name.clone();
        let error = rename_registered(&mut configs[1], &taken)
            .expect_err("a name already in use must be refused");

        assert!(
            error.to_string().contains("already exists"),
            "unexpected error: {error}"
        );
    }

    /// The configuration id is what a licence is pinned to, so it must survive
    /// restarts. It previously did not: the first run keyed the registry on a
    /// non-canonical path (config.toml did not exist yet) and the second on a
    /// canonical one, producing a second row and a new id.
    #[test]
    fn the_configuration_id_is_stable_across_restarts() {
        let temp = tempfile::tempdir().unwrap();
        let registry = temp.path().join(REGISTRY_FILE);
        let root = temp.path().join("workspace");
        std::fs::create_dir_all(&root).unwrap();

        let identity = |()| {
            let mut config = Config::default();
            config.paths.state = root.to_string_lossy().into_owned();
            config.paths.tickets = root.join("tickets").to_string_lossy().into_owned();
            register_in(&mut config, &registry).unwrap();
            config.profile
        };

        let first = identity(());
        let second = identity(());
        let third = identity(());

        assert_eq!(first.id, second.id);
        assert_eq!(second.id, third.id);
        assert_eq!(first.name, third.name, "a stable id keeps a stable name");
    }

    /// A first registration must mint its own id rather than trust one that
    /// arrived in the configuration file.
    #[test]
    fn a_first_registration_ignores_an_id_supplied_by_the_config_file() {
        let temp = tempfile::tempdir().unwrap();
        let registry = temp.path().join(REGISTRY_FILE);
        let root = temp.path().join("workspace");
        std::fs::create_dir_all(&root).unwrap();

        let planted = Uuid::new_v4();
        let mut config = Config::default();
        config.paths.state = root.to_string_lossy().into_owned();
        config.paths.tickets = root.join("tickets").to_string_lossy().into_owned();
        config.profile.id = planted;
        register_in(&mut config, &registry).unwrap();

        assert_ne!(
            config.profile.id, planted,
            "a planted id must not become this configuration's identity"
        );
    }

    /// Two processes registering at once must both succeed with distinct
    /// names: `operator` runs this on every invocation, so a lost race exits
    /// the process before it can serve anything.
    #[test]
    fn concurrent_registration_of_two_configs_assigns_distinct_names() {
        let temp = tempfile::tempdir().unwrap();
        let registry = temp.path().join(REGISTRY_FILE);
        open_registry(&registry).unwrap();

        let configs: Vec<_> = (0..2)
            .map(|i| {
                let root = temp.path().join(format!("workspace-{i}"));
                std::fs::create_dir_all(&root).unwrap();
                let mut config = Config::default();
                config.paths.state = root.to_string_lossy().into_owned();
                config.paths.tickets = root.join("tickets").to_string_lossy().into_owned();
                config
            })
            .collect();

        let results: Vec<_> = std::thread::scope(|scope| {
            let handles: Vec<_> = configs
                .into_iter()
                .map(|mut config| {
                    let registry = registry.clone();
                    scope
                        .spawn(move || register_in(&mut config, &registry).map(|()| config.profile))
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });

        let identities: Vec<_> = results
            .into_iter()
            .map(|r| r.expect("both registrations must succeed"))
            .collect();
        assert_ne!(identities[0].id, identities[1].id);
        assert_ne!(identities[0].name, identities[1].name);
    }

    #[test]
    fn named_profiles_have_isolated_storage_and_survive_reload() {
        let temp = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = temp.path().join("state").to_string_lossy().into_owned();
        config.paths.tickets = temp.path().join("tickets").to_string_lossy().into_owned();
        config.profile_registry = Some(temp.path().join(REGISTRY_FILE));
        let primary = ApiState::new(config.clone(), config.tickets_path());
        let profiles = ServerProfiles::open(primary.clone()).unwrap();
        let a = profiles.create("first").unwrap();
        let b = profiles.create("second").unwrap();
        assert_ne!(a.id, b.id);
        assert!(!a.initialized);
        assert!(profiles.create("first").is_err());
        assert_ne!(
            profiles.state(a.id).unwrap().tickets_path,
            profiles.state(b.id).unwrap().tickets_path
        );
        drop(profiles);
        let reopened = ServerProfiles::open(primary).unwrap();
        assert_eq!(reopened.state(a.id).unwrap().config().profile.name, "first");
    }
}
