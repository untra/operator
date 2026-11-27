use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::agents::vscode_types::{VsCodeExistsResponse, VsCodeSessionInfo};
#[cfg(unix)]
use anyhow::ensure;
use anyhow::{Context, Result};

#[cfg(unix)]
fn unix_registration_safe(
    home: &Path,
    path: &Path,
    private: bool,
    directory: bool,
) -> Result<bool> {
    use std::os::unix::fs::MetadataExt;

    let meta = std::fs::symlink_metadata(path)?;
    Ok((if directory {
        meta.is_dir()
    } else {
        meta.is_file()
    }) && meta.uid() == std::fs::metadata(home)?.uid()
        && meta.mode() & if private { 0o077 } else { 0o022 } == 0)
}

fn registrations(home: &Path, workspaces: &[PathBuf]) -> Result<Vec<VsCodeSessionInfo>> {
    let parent = home.join(".operator");
    let dir = parent.join("vscode-sessions");
    #[cfg(unix)]
    ensure!(
        unix_registration_safe(home, &parent, false, true)?
            && unix_registration_safe(home, &dir, true, true)?,
        "Invalid terminal registration directory"
    );
    #[cfg(windows)]
    use crate::windows_files::{CurrentUser, RegistrationObject};
    #[cfg(windows)]
    let user = CurrentUser::new()?;
    #[cfg(windows)]
    let _directories = [
        (&parent, RegistrationObject::Parent),
        (&dir, RegistrationObject::Directory),
    ]
    .into_iter()
    .map(|(path, object)| {
        user.open_registration(path, object)?
            .context("Invalid terminal registration directory")
    })
    .collect::<Result<Vec<_>>>()?;
    #[cfg(not(any(unix, windows)))]
    anyhow::bail!("Terminal registration ownership checks are unavailable");

    let workspaces: Vec<_> = workspaces
        .iter()
        .filter_map(|p| p.canonicalize().ok())
        .collect();
    let mut sessions = Vec::new();
    for file in std::fs::read_dir(dir)? {
        let path = file?.path();
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        #[cfg(unix)]
        let file = unix_registration_safe(home, &path, true, false)
            .unwrap_or(false)
            .then(|| std::fs::File::open(&path).ok())
            .flatten();
        #[cfg(windows)]
        let file = user
            .open_registration(&path, RegistrationObject::File)
            .ok()
            .flatten();
        let Some(session) = file.and_then(|file| {
            serde_json::from_reader::<_, VsCodeSessionInfo>(std::io::BufReader::new(file)).ok()
        }) else {
            continue;
        };
        if Path::new(&session.workspace)
            .canonicalize()
            .is_ok_and(|p| workspaces.contains(&p))
        {
            sessions.push(session);
        }
    }
    Ok(sessions)
}

pub async fn focus(workspaces: Vec<PathBuf>, name: String) -> Result<()> {
    let home = dirs::home_dir().context("No home for terminal registration")?;
    let sessions = tokio::task::spawn_blocking(move || registrations(&home, &workspaces)).await??;
    let client = crate::http_client::client_builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(2))
        .build()?;
    for session in sessions {
        let mut url = url::Url::parse(&format!("http://127.0.0.1:{}", session.port))?;
        url.path_segments_mut()
            .expect("HTTP base")
            .extend(["terminal", name.as_str(), "exists"]);
        let Ok(response) = client
            .get(url.clone())
            .bearer_auth(&session.bearer)
            .send()
            .await
        else {
            continue;
        };
        if response.status().is_success()
            && response
                .json::<VsCodeExistsResponse>()
                .await
                .is_ok_and(|s| s.exists)
        {
            url.path_segments_mut()
                .expect("HTTP base")
                .pop()
                .push("focus");
            client
                .post(url)
                .bearer_auth(&session.bearer)
                .send()
                .await?
                .error_for_status()?;
            return Ok(());
        }
    }
    anyhow::bail!("No active editor window owns this terminal")
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_registrations_filter_private_files_for_requested_workspace() {
        let home = tempfile::tempdir().unwrap();
        let project = home.path().join("project");
        std::fs::create_dir(&project).unwrap();
        let dir = home.path().join(".operator/vscode-sessions");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
        let file = dir.join("window.json");
        let session = VsCodeSessionInfo {
            bearer: "test".into(),
            wrapper: "vscode".into(),
            workspace: project.to_string_lossy().into_owned(),
            port: 7009,
            pid: 1,
            version: "fixture".into(),
            started_at: String::new(),
        };
        std::fs::write(&file, serde_json::to_vec(&session).unwrap()).unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(registrations(home.path(), &[dir]).unwrap().is_empty());
        assert_eq!(
            registrations(home.path(), std::slice::from_ref(&project))
                .unwrap()
                .len(),
            1
        );
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(registrations(home.path(), &[project]).unwrap().is_empty());
    }
}
