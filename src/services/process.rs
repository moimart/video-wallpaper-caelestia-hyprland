use std::process::{Command, Output};

/// Run a command, capture output, and log stderr on failure.
pub fn run(cmd: &str, args: &[&str]) -> Result<Output, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("{cmd}: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::debug!("{cmd} stderr: {stderr}");
    }

    Ok(output)
}

/// Spawn a detached process (don't wait for it).
pub fn spawn(cmd: &str, args: &[impl AsRef<std::ffi::OsStr>]) -> Result<u32, String> {
    let child = Command::new(cmd)
        .args(args)
        .spawn()
        .map_err(|e| format!("Failed to spawn {cmd}: {e}"))?;

    Ok(child.id())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_succeeds() {
        let output = run("echo", &["hello"]).unwrap();
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("hello"));
    }

    #[test]
    fn run_captures_failure() {
        let output = run("false", &[]);
        assert!(output.is_ok()); // returns Ok even on non-zero exit
        assert!(!output.unwrap().status.success());
    }

    #[test]
    fn run_reports_missing_command() {
        let result = run("nonexistent_command_xyz", &[]);
        assert!(result.is_err());
    }
}
