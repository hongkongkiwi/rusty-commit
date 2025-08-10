use anyhow::{Context, Result};
use std::fs;
use std::process::{Command, Stdio};
use std::time::Duration;

pub struct HookOptions<'a> {
    pub name: &'a str,
    pub commands: Vec<String>,
    pub strict: bool,
    pub timeout: Duration,
    pub envs: Vec<(&'a str, String)>,
}

pub fn run_hooks(opts: HookOptions) -> Result<()> {
    for (idx, cmd) in opts.commands.iter().enumerate() {
        let mut command = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.arg("/C").arg(cmd);
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-lc").arg(cmd);
            c
        };

        for (k, v) in &opts.envs {
            command.env(k, v);
        }

        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());

        let mut child = command
            .spawn()
            .with_context(|| format!("Failed to start {} hook {}: {}", opts.name, idx + 1, cmd))?;

        // Simple timeout: wait with polling
        let start = std::time::Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if start.elapsed() > opts.timeout {
                // Best-effort: attempt to terminate the child process
                let _ = child.kill();
                return Err(anyhow::anyhow!(
                    "{} hook timed out after {:?} while running: {}",
                    opts.name,
                    opts.timeout,
                    cmd
                ));
            }
            std::thread::sleep(Duration::from_millis(100));
        };

        if !status.success() {
            let msg = format!(
                "{} hook failed (exit status {:?}) for command: {}",
                opts.name,
                status.code(),
                cmd
            );
            if opts.strict {
                return Err(anyhow::anyhow!(msg));
            } else {
                eprintln!("Warning: {}", msg);
            }
        }
    }

    Ok(())
}

/// Utility to write/read a temporary commit message file for hooks to modify
pub fn write_temp_commit_file(initial: &str) -> Result<std::path::PathBuf> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("rco-commit-msg.txt");
    fs::write(&path, initial)?;
    // Keep tempdir alive by leaking it; file will be short-lived
    std::mem::forget(dir);
    Ok(path)
}
