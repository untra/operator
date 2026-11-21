use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use operator::config::Config;
use operator::ownership::Ownership;
use operator::ownership::ResourceGuard;

const RESOURCE_ENV: &str = "OPERATOR_TEST_OWNERSHIP_RESOURCE";
const READY: &str = "ownership-ready";
const IDLE_ENV: &str = "OPERATOR_TEST_IDLE_CHILD";
const GRANDCHILD_ENV: &str = "OPERATOR_TEST_SPAWN_IDLE_CHILD";

#[test]
fn test_lock_holder() {
    let path = std::env::var_os(RESOURCE_ENV);
    if path.is_none() && std::env::var_os(IDLE_ENV).is_none() {
        return;
    }
    let _guard = path.map(|path| ResourceGuard::state(std::path::Path::new(&path)).unwrap());
    let mut child = std::env::var_os(GRANDCHILD_ENV).map(|_| {
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "test_lock_holder", "--nocapture"])
            .env(IDLE_ENV, "1")
            .env_remove(RESOURCE_ENV)
            .env_remove(GRANDCHILD_ENV)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .spawn()
            .unwrap()
    });
    println!("{READY}");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    if let Some(child) = child.as_mut() {
        child.wait().unwrap();
    }
}

#[test]
fn test_state_lock_excludes_process_and_releases_after_exit() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("state");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "test_lock_holder", "--nocapture"])
        .env(RESOURCE_ENV, &path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    loop {
        line.clear();
        assert_ne!(output.read_line(&mut line).unwrap(), 0);
        if line.trim() == READY {
            break;
        }
    }
    let error = ResourceGuard::state(&path).unwrap_err();
    assert!(error.to_string().contains("owned"), "{error:#}");
    child.stdin.take().unwrap().write_all(b"stop\n").unwrap();
    assert!(child.wait().unwrap().success());
    ResourceGuard::state(&path).unwrap();
}

#[test]
fn test_aliases_share_a_lock_and_independent_states_do_not() {
    let temp = tempfile::tempdir().unwrap();
    let state = temp.path().join("state");
    let _guard = ResourceGuard::state(&state).unwrap();
    assert!(ResourceGuard::state(&state.join(".")).is_err());
    ResourceGuard::state(&temp.path().join("independent")).unwrap();
    #[cfg(unix)]
    {
        let alias = temp.path().join("alias");
        std::os::unix::fs::symlink(&state, &alias).unwrap();
        assert!(ResourceGuard::state(&alias).is_err());
    }
}

#[test]
fn test_registry_lock_survives_registry_replacement() {
    let temp = tempfile::tempdir().unwrap();
    let registry = temp.path().join("profiles.sqlite");
    let _guard = ResourceGuard::registry(&registry).unwrap();
    std::fs::write(&registry, b"old").unwrap();
    let replacement = temp.path().join("replacement");
    std::fs::write(&replacement, b"new").unwrap();
    std::fs::rename(replacement, &registry).unwrap();
    assert!(ResourceGuard::registry(&registry).is_err());
}

#[test]
fn test_child_does_not_extend_parent_ownership() {
    let temp = tempfile::tempdir().unwrap();
    let guard = ResourceGuard::state(temp.path()).unwrap();
    let child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "test_lock_holder", "--nocapture"])
        .env(IDLE_ENV, "1")
        .env_remove(RESOURCE_ENV)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut child = OwnerProcess(child);
    let mut output = BufReader::new(child.0.stdout.take().unwrap());
    let mut line = String::new();
    loop {
        line.clear();
        assert_ne!(output.read_line(&mut line).unwrap(), 0);
        if line.trim() == READY {
            break;
        }
    }
    drop(guard);
    ResourceGuard::state(temp.path()).unwrap();
    assert!(child.0.try_wait().unwrap().is_none());
}

#[test]
fn test_process_death_releases_ownership_with_a_surviving_child() {
    let temp = tempfile::tempdir().unwrap();
    let child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "test_lock_holder", "--nocapture"])
        .env(RESOURCE_ENV, temp.path())
        .env(GRANDCHILD_ENV, "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut holder = OwnerProcess(child);
    let mut output = BufReader::new(holder.0.stdout.take().unwrap());
    let mut line = String::new();
    let mut ready = 0;
    while ready < 2 {
        line.clear();
        assert_ne!(output.read_line(&mut line).unwrap(), 0);
        if line.trim() == READY {
            ready += 1;
        }
    }
    assert!(ResourceGuard::state(temp.path()).is_err());
    holder.0.kill().unwrap();
    holder.0.wait().unwrap();
    ResourceGuard::state(temp.path()).unwrap();
    drop(holder.0.stdin.take());
    while output.read_line(&mut line).unwrap() != 0 {
        line.clear();
    }
}

fn config_in(root: &std::path::Path) -> Config {
    let mut config = Config::default();
    config.paths.state = root.join("state").to_string_lossy().into_owned();
    config.paths.tickets = root.join("tickets").to_string_lossy().into_owned();
    config.profile_registry = Some(root.join("profiles.sqlite"));
    config
}

#[test]
fn test_later_lock_failure_releases_new_registry_and_state_guards() {
    let temp = tempfile::tempdir().unwrap();
    let mut config = config_in(temp.path());
    config.paths.state = temp.path().join("a-state").to_string_lossy().into_owned();
    config.server_auth_path = Some(temp.path().join("z-auth"));
    let _conflict = ResourceGuard::state(&config.auth_state_path()).unwrap();
    assert!(Ownership::acquire(&config).is_err());
    ResourceGuard::registry(config.profile_registry.as_ref().unwrap()).unwrap();
    ResourceGuard::state(&config.state_path()).unwrap();
}

#[test]
fn test_shared_registry_excludes_distinct_states() {
    let temp = tempfile::tempdir().unwrap();
    let config = config_in(temp.path());
    let _conflict = ResourceGuard::registry(config.profile_registry.as_ref().unwrap()).unwrap();
    assert!(Ownership::acquire(&config).is_err());
    assert!(!config.state_path().exists());
}

#[test]
fn test_shared_auth_directory_is_owned_before_initialization() {
    let temp = tempfile::tempdir().unwrap();
    let mut config = config_in(temp.path());
    config.server_auth_path = Some(temp.path().join("shared-auth"));
    let _conflict = ResourceGuard::state(&config.auth_state_path()).unwrap();
    assert!(operator::rest::ApiState::try_new(config.clone(), config.tickets_path()).is_err());
    assert!(!config.tickets_path().exists());
    assert_eq!(
        std::fs::read_dir(config.auth_state_path()).unwrap().count(),
        1
    );
}

#[test]
fn test_offline_auth_open_refuses_an_owned_state() {
    let temp = tempfile::tempdir().unwrap();
    let _guard = ResourceGuard::state(temp.path()).unwrap();
    assert!(operator::auth::store::AuthStore::open(temp.path()).is_err());
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
}

#[test]
fn test_independent_instances_and_live_path_change_rejection() {
    let temp = tempfile::tempdir().unwrap();
    let config = config_in(&temp.path().join("first"));
    let first = Ownership::acquire(&config).unwrap();
    let second_config = config_in(&temp.path().join("second"));
    let _second = Ownership::acquire(&second_config).unwrap();
    first.validate(&config).unwrap();
    let mut changed = config;
    changed.paths.state = second_config.paths.state;
    assert!(first.validate(&changed).is_err());
}

#[test]
fn test_auth_initialization_failure_releases_all_ownership() {
    let temp = tempfile::tempdir().unwrap();
    let config = config_in(temp.path());
    std::fs::create_dir_all(config.state_path()).unwrap();
    std::fs::create_dir(
        config
            .state_path()
            .join(operator::auth::store::AUTH_DB_FILENAME),
    )
    .unwrap();
    let result = operator::rest::ApiState::try_new(config.clone(), config.tickets_path());
    assert!(result.is_err());
    ResourceGuard::registry(config.profile_registry.as_ref().unwrap()).unwrap();
    ResourceGuard::state(&config.state_path()).unwrap();
}

#[tokio::test]
async fn test_listener_bind_failure_does_not_publish_and_releases_ownership() {
    let temp = tempfile::tempdir().unwrap();
    let config = config_in(temp.path());
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let state = operator::rest::ApiState::try_new(config.clone(), config.tickets_path()).unwrap();
    assert!(
        operator::rest::serve(state, listener.local_addr().unwrap().port(), false)
            .await
            .is_err()
    );
    assert!(!config
        .tickets_path()
        .join("operator/api-session.json")
        .exists());
    ResourceGuard::registry(config.profile_registry.as_ref().unwrap()).unwrap();
    ResourceGuard::state(&config.state_path()).unwrap();
}

#[test]
fn test_auth_store_clone_retains_the_entire_runtime_lease() {
    let temp = tempfile::tempdir().unwrap();
    let config = config_in(temp.path());
    let state = operator::rest::ApiState::try_new(config.clone(), config.tickets_path()).unwrap();
    let store = state.auth.store.clone();
    drop(state);
    assert!(ResourceGuard::registry(config.profile_registry.as_ref().unwrap()).is_err());
    assert!(ResourceGuard::state(&config.state_path()).is_err());
    drop(store);
    ResourceGuard::registry(config.profile_registry.as_ref().unwrap()).unwrap();
    ResourceGuard::state(&config.state_path()).unwrap();
}

struct OwnerProcess(std::process::Child);

impl Drop for OwnerProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn operator_command(config: &std::path::Path, registry: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_operator"));
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("OPERATOR_") {
            command.env_remove(key);
        }
    }
    command
        .arg("--config")
        .arg(config)
        .env("OPERATOR_PROFILE_REGISTRY", registry);
    command
}

#[test]
fn test_api_owner_blocks_cli_mcp_acp_and_other_ports_until_process_exit() {
    let temp = tempfile::tempdir().unwrap();
    let mut config = config_in(temp.path());
    config.rest_api.enabled = true;
    config.rest_api.host = "127.0.0.1".into();
    let path = temp.path().join("config.toml");
    std::fs::write(&path, toml::to_string(&config).unwrap()).unwrap();
    let registry = config.profile_registry.as_ref().unwrap();
    let child = operator_command(&path, registry)
        .args(["api", "--port", "0"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut owner = OwnerProcess(child);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let session = config.tickets_path().join("operator/api-session.json");
    loop {
        assert!(
            owner.0.try_wait().unwrap().is_none(),
            "owner exited before readiness"
        );
        if let Some(port) = std::fs::read(&session)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
            .and_then(|session| session["port"].as_u64())
            .and_then(|port| u16::try_from(port).ok())
        {
            if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
                break;
            }
        }
        assert!(
            std::time::Instant::now() < deadline,
            "owner did not become ready"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let token_path = config
        .auth_state_path()
        .join(operator::auth::local::LOCAL_TOKEN_FILENAME);
    let token_before = std::fs::read(&token_path).unwrap();
    let registry_before = std::fs::read(registry).unwrap();
    let state_before = std::fs::read(config.state_path().join("state.json")).unwrap();
    for args in [
        vec![],
        vec!["queue"],
        vec!["mcp"],
        vec!["acp"],
        vec!["auth", "status"],
        vec!["api", "--port", "0"],
    ] {
        let output = operator_command(&path, registry)
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("already owned"),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(std::fs::read(&token_path).unwrap(), token_before);
        assert_eq!(std::fs::read(registry).unwrap(), registry_before);
        assert_eq!(
            std::fs::read(config.state_path().join("state.json")).unwrap(),
            state_before
        );
        assert!(output.stdout.is_empty());
    }
    let other_registry = temp.path().join("other-registry.sqlite");
    let conflict = operator_command(&path, &other_registry)
        .arg("queue")
        .output()
        .unwrap();
    assert!(!conflict.status.success());
    assert!(String::from_utf8_lossy(&conflict.stderr).contains("already owned"));
    assert!(!other_registry.exists());

    let independent = config_in(&temp.path().join("independent"));
    let independent_path = temp.path().join("independent.toml");
    std::fs::write(&independent_path, toml::to_string(&independent).unwrap()).unwrap();
    let conflict = operator_command(&independent_path, registry)
        .arg("queue")
        .output()
        .unwrap();
    assert!(!conflict.status.success());
    assert!(!independent.state_path().exists());
    let independent_result = operator_command(
        &independent_path,
        independent.profile_registry.as_ref().unwrap(),
    )
    .arg("queue")
    .output()
    .unwrap();
    assert!(
        independent_result.status.success(),
        "{}",
        String::from_utf8_lossy(&independent_result.stderr)
    );
    owner.0.kill().unwrap();
    owner.0.wait().unwrap();
    assert!(
        session.exists(),
        "forced exit should leave stale discovery metadata"
    );
    ResourceGuard::registry(registry).unwrap();
    ResourceGuard::state(&config.state_path()).unwrap();
}
