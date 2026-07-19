use std::{path::PathBuf, process::Stdio};
use tokio::process::{Child, Command};

#[derive(Debug)]
pub enum ProcessError {
    NotFound,
    Start(String),
}

pub async fn discover(custom: Option<&str>) -> Result<PathBuf, ProcessError> {
    if let Some(path) = custom {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return Ok(candidate);
        }
        return Err(ProcessError::NotFound);
    }
    if Command::new("codex")
        .arg("--version")
        .output()
        .await
        .is_ok()
    {
        return Ok(PathBuf::from("codex"));
    }
    let output = Command::new("where.exe")
        .arg("codex")
        .output()
        .await
        .map_err(|_| ProcessError::NotFound)?;
    let path = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(PathBuf::from);
    path.filter(|p| p.is_file()).ok_or(ProcessError::NotFound)
}

pub async fn spawn(executable: PathBuf) -> Result<Child, ProcessError> {
    let mut command = Command::new(executable);
    command
        .arg("app-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(windows)]
    {
        command.creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW);
    }
    command
        .spawn()
        .map_err(|e| ProcessError::Start(format!("Unable to start Codex CLI: {e}")))
}
