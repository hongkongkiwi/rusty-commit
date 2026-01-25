use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

pub struct HookOptions<'a> {
    pub name: &'a str,
    pub commands: Vec<String>,
    pub strict: bool,
    pub timeout: Duration,
    pub envs: Vec<(&'a str, String)>,
}

/// Safely parse a command string into executable and arguments.
/// Returns None if the command is empty or only whitespace.
fn parse_command(cmd: &str) -> Option<(String, Vec<String>)> {
    let cmd = cmd.trim();
    if cmd.is_empty() {
        return None;
    }

    // Use shell-like parsing: first word is command, rest are args
    let mut parts = cmd.split_whitespace();
    let executable = parts.next()?.to_string();
    let args: Vec<String> = parts.map(|s| s.to_string()).collect();

    Some((executable, args))
}

pub fn run_hooks(opts: HookOptions) -> Result<()> {
    for (idx, cmd) in opts.commands.iter().enumerate() {
        // Parse command into executable and arguments
        let (executable, args) = match parse_command(cmd) {
            Some(parts) => parts,
            None => {
                eprintln!("Warning: Empty command in {} hook {}", opts.name, idx + 1);
                continue;
            }
        };

        // Security: warn about potentially dangerous commands
        let executable_lower = executable.to_lowercase();
        if executable_lower == "sh"
            || executable_lower == "bash"
            || executable_lower == "cmd"
            || executable_lower == "powershell"
        {
            eprintln!(
                "Warning: Shell execution in {} hook {} is deprecated for security reasons. \
                Consider using direct command execution instead: {}",
                opts.name,
                idx + 1,
                cmd
            );
        }

        let mut command = Command::new(&executable);
        command.args(&args);

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

/// Utility to write/read a temporary commit message file for hooks to modify.
/// Uses NamedTempFile to avoid memory leaks while keeping the file accessible.
pub fn write_temp_commit_file(initial: &str) -> Result<PathBuf> {
    let mut temp_file = tempfile::NamedTempFile::new()?;
    temp_file.write_all(initial.as_bytes())?;
    // Persist the file so it survives beyond this function
    let path = temp_file.into_temp_path();
    let final_path = std::env::temp_dir().join(format!("rco-commit-{:}.txt", std::process::id()));
    path.persist(&final_path)?;
    Ok(final_path)
}

/// Cleanup function for temp commit file - call this after commit is done
#[allow(dead_code)]
pub fn cleanup_temp_commit_file(path: &PathBuf) {
    let _ = fs::remove_file(path);
}
