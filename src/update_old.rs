use anyhow::{Context, Result};
use colored::*;
use reqwest;
use semver::Version;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum InstallMethod {
    Homebrew,
    Cargo,
    Deb,
    Rpm,
    Binary,
    Snap,
    Unknown,
}

#[derive(Debug)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub install_method: InstallMethod,
    pub executable_path: PathBuf,
    pub needs_update: bool,
}

/// Detect how rusty-commit was installed
pub fn detect_install_method() -> Result<InstallMethod> {
    let exe_path = env::current_exe().context("Failed to get current executable path")?;
    let exe_str = exe_path.to_string_lossy();
    
    // Check for Homebrew installation
    if exe_str.contains("/Cellar/") || exe_str.contains("homebrew") {
        return Ok(InstallMethod::Homebrew);
    }
    
    // Check for Cargo installation
    if exe_str.contains(".cargo/bin") {
        return Ok(InstallMethod::Cargo);
    }
    
    // Check for Snap installation
    if exe_str.contains("/snap/") {
        return Ok(InstallMethod::Snap);
    }
    
    // Check for system package manager installations
    if exe_str.starts_with("/usr/bin/") || exe_str.starts_with("/usr/local/bin/") {
        // Try to detect package manager
        if Path::new("/etc/debian_version").exists() {
            // Check if installed via dpkg
            let output = Command::new("dpkg")
                .args(&["-S", &exe_path.to_string_lossy()])
                .output();
            
            if let Ok(output) = output {
                if output.status.success() {
                    return Ok(InstallMethod::Deb);
                }
            }
        }
        
        if Path::new("/etc/redhat-release").exists() || Path::new("/etc/fedora-release").exists() {
            // Check if installed via rpm
            let output = Command::new("rpm")
                .args(&["-qf", &exe_path.to_string_lossy()])
                .output();
            
            if let Ok(output) = output {
                if output.status.success() {
                    return Ok(InstallMethod::Rpm);
                }
            }
        }
        
        // Likely a binary installation
        return Ok(InstallMethod::Binary);
    }
    
    // Check if it's in a typical binary install location
    if exe_str.contains("/usr/local/bin/") || exe_str.contains("/opt/") {
        return Ok(InstallMethod::Binary);
    }
    
    Ok(InstallMethod::Unknown)
}

/// Get the latest version from GitHub releases
pub async fn get_latest_version() -> Result<String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/hongkongkiwi/rusty-commit/releases/latest")
        .header("User-Agent", "rusty-commit")
        .send()
        .await
        .context("Failed to fetch latest release")?;
    
    let release: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse release JSON")?;
    
    let tag_name = release["tag_name"]
        .as_str()
        .context("Failed to get tag_name from release")?;
    
    // Remove 'v' prefix if present
    Ok(tag_name.trim_start_matches('v').to_string())
}

/// Check if an update is available
pub async fn check_for_update() -> Result<UpdateInfo> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let latest_version = get_latest_version().await?;
    let install_method = detect_install_method()?;
    let executable_path = env::current_exe()?;
    
    let current = Version::parse(&current_version)?;
    let latest = Version::parse(&latest_version)?;
    
    let needs_update = latest > current;
    
    Ok(UpdateInfo {
        current_version,
        latest_version,
        install_method,
        executable_path,
        needs_update,
    })
}

/// Update using Homebrew
async fn update_homebrew() -> Result<()> {
    println!("{}", "Updating via Homebrew...".blue());
    
    // Update Homebrew
    let output = Command::new("brew")
        .args(&["update"])
        .output()
        .context("Failed to run brew update")?;
    
    if !output.status.success() {
        anyhow::bail!("brew update failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Upgrade rusty-commit
    let output = Command::new("brew")
        .args(&["upgrade", "rusty-commit"])
        .output()
        .context("Failed to run brew upgrade")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("already installed") {
            println!("{}", "Already up to date!".green());
            return Ok(());
        }
        anyhow::bail!("brew upgrade failed: {}", stderr);
    }
    
    println!("{}", "Successfully updated via Homebrew!".green());
    Ok(())
}

/// Update using Cargo
async fn update_cargo() -> Result<()> {
    println!("{}", "Updating via Cargo...".blue());
    
    let output = Command::new("cargo")
        .args(&["install", "rusty-commit", "--force", "--features", "secure-storage"])
        .output()
        .context("Failed to run cargo install")?;
    
    if !output.status.success() {
        anyhow::bail!("cargo install failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    println!("{}", "Successfully updated via Cargo!".green());
    Ok(())
}

/// Update Debian package
async fn update_deb(version: &str) -> Result<()> {
    println!("{}", "Updating via apt/dpkg...".blue());
    
    let arch = get_system_arch()?;
    let deb_arch = match arch.as_str() {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "armv7" => "armhf",
        _ => anyhow::bail!("Unsupported architecture for .deb: {}", arch),
    };
    
    let url = format!(
        "https://github.com/hongkongkiwi/rusty-commit/releases/download/v{}/rusty-commit_{}_{}.deb",
        version, version, deb_arch
    );
    
    // Download to temp file
    let temp_path = download_to_temp(&url, "rusty-commit.deb").await?;
    
    // Install with dpkg or apt
    let output = if which::which("apt-get").is_ok() {
        Command::new("sudo")
            .args(&["apt-get", "install", "-y", &temp_path.to_string_lossy()])
            .output()
            .context("Failed to run apt-get install")?
    } else {
        Command::new("sudo")
            .args(&["dpkg", "-i", &temp_path.to_string_lossy()])
            .output()
            .context("Failed to run dpkg -i")?
    };
    
    // Clean up temp file
    let _ = fs::remove_file(temp_path);
    
    if !output.status.success() {
        anyhow::bail!("Package installation failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    println!("{}", "Successfully updated via package manager!".green());
    Ok(())
}

/// Update RPM package
async fn update_rpm(version: &str) -> Result<()> {
    println!("{}", "Updating via rpm/dnf/yum...".blue());
    
    let arch = get_system_arch()?;
    let rpm_arch = match arch.as_str() {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => anyhow::bail!("Unsupported architecture for .rpm: {}", arch),
    };
    
    let url = format!(
        "https://github.com/hongkongkiwi/rusty-commit/releases/download/v{}/rusty-commit-{}-1.{}.rpm",
        version, version, rpm_arch
    );
    
    // Download to temp file
    let temp_path = download_to_temp(&url, "rusty-commit.rpm").await?;
    
    // Install with dnf, yum, or rpm
    let output = if which::which("dnf").is_ok() {
        Command::new("sudo")
            .args(&["dnf", "install", "-y", &temp_path.to_string_lossy()])
            .output()
            .context("Failed to run dnf install")?
    } else if which::which("yum").is_ok() {
        Command::new("sudo")
            .args(&["yum", "install", "-y", &temp_path.to_string_lossy()])
            .output()
            .context("Failed to run yum install")?
    } else {
        Command::new("sudo")
            .args(&["rpm", "-Uvh", &temp_path.to_string_lossy()])
            .output()
            .context("Failed to run rpm -Uvh")?
    };
    
    // Clean up temp file
    let _ = fs::remove_file(temp_path);
    
    if !output.status.success() {
        anyhow::bail!("Package installation failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    println!("{}", "Successfully updated via package manager!".green());
    Ok(())
}

/// Update binary installation
async fn update_binary(version: &str, exe_path: &Path) -> Result<()> {
    println!("{}", "Updating binary installation...".blue());
    
    let os = get_system_os()?;
    let arch = get_system_arch()?;
    
    let archive_name = match (os.as_str(), arch.as_str()) {
        ("linux", "x86_64") => "rustycommit-linux-x86_64.tar.gz",
        ("linux", "aarch64") => "rustycommit-linux-aarch64.tar.gz",
        ("linux", "armv7") => "rustycommit-linux-armv7.tar.gz",
        ("macos", "x86_64") => "rustycommit-macos-x86_64.tar.gz",
        ("macos", "aarch64") => "rustycommit-macos-aarch64.tar.gz",
        ("windows", "x86_64") => "rustycommit-windows-x86_64.zip",
        ("windows", "i686") => "rustycommit-windows-i686.zip",
        _ => anyhow::bail!("Unsupported OS/architecture: {}-{}", os, arch),
    };
    
    let url = format!(
        "https://github.com/hongkongkiwi/rusty-commit/releases/download/v{}/{}",
        version, archive_name
    );
    
    // Download archive
    let temp_archive = download_to_temp(&url, archive_name).await?;
    
    // Extract to temp directory
    let temp_dir = tempfile::tempdir()?;
    
    if archive_name.ends_with(".zip") {
        // Extract zip
        let output = Command::new("unzip")
            .args(&["-q", &temp_archive.to_string_lossy(), "-d", &temp_dir.path().to_string_lossy()])
            .output()
            .context("Failed to extract zip archive")?;
        
        if !output.status.success() {
            anyhow::bail!("Failed to extract archive: {}", String::from_utf8_lossy(&output.stderr));
        }
    } else {
        // Extract tar.gz
        let output = Command::new("tar")
            .args(&["xzf", &temp_archive.to_string_lossy(), "-C", &temp_dir.path().to_string_lossy()])
            .output()
            .context("Failed to extract tar archive")?;
        
        if !output.status.success() {
            anyhow::bail!("Failed to extract archive: {}", String::from_utf8_lossy(&output.stderr));
        }
    }
    
    // Find the binary in extracted files
    let binary_name = if cfg!(windows) { "rco.exe" } else { "rco" };
    let new_binary = temp_dir.path().join(binary_name);
    
    if !new_binary.exists() {
        anyhow::bail!("Binary not found in archive");
    }
    
    // Create backup of current binary
    let backup_path = exe_path.with_extension("bak");
    fs::copy(exe_path, &backup_path)
        .context("Failed to create backup of current binary")?;
    
    // Replace binary (may need sudo)
    let needs_sudo = !is_writable(exe_path)?;
    
    if needs_sudo {
        println!("{}", "Administrator privileges required to update binary...".yellow());
        
        // Use sudo to move the new binary
        let output = Command::new("sudo")
            .args(&["mv", &new_binary.to_string_lossy(), &exe_path.to_string_lossy()])
            .output()
            .context("Failed to run sudo mv")?;
        
        if !output.status.success() {
            // Restore backup
            let _ = fs::rename(&backup_path, exe_path);
            anyhow::bail!("Failed to replace binary: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        // Set executable permissions
        let _ = Command::new("sudo")
            .args(&["chmod", "+x", &exe_path.to_string_lossy()])
            .output();
    } else {
        // Direct replacement
        fs::copy(&new_binary, exe_path)
            .context("Failed to replace binary")?;
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(exe_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(exe_path, perms)?;
        }
    }
    
    // Remove backup
    let _ = fs::remove_file(backup_path);
    
    // Clean up temp files
    let _ = fs::remove_file(temp_archive);
    
    println!("{}", "Successfully updated binary!".green());
    println!("{}", "Please restart your terminal or run 'hash -r' to use the new version.".yellow());
    Ok(())
}

/// Update Snap package
async fn update_snap() -> Result<()> {
    println!("{}", "Updating via Snap...".blue());
    
    let output = Command::new("sudo")
        .args(&["snap", "refresh", "rusty-commit"])
        .output()
        .context("Failed to run snap refresh")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("has no updates available") {
            println!("{}", "Already up to date!".green());
            return Ok(());
        }
        anyhow::bail!("snap refresh failed: {}", stderr);
    }
    
    println!("{}", "Successfully updated via Snap!".green());
    Ok(())
}

/// Perform the update based on installation method
pub async fn perform_update(info: &UpdateInfo) -> Result<()> {
    if !info.needs_update {
        println!("{}", "Already running the latest version!".green());
        return Ok(());
    }
    
    println!(
        "{}",
        format!(
            "Updating from v{} to v{}...",
            info.current_version, info.latest_version
        ).blue()
    );
    
    match info.install_method {
        InstallMethod::Homebrew => update_homebrew().await,
        InstallMethod::Cargo => update_cargo().await,
        InstallMethod::Deb => update_deb(&info.latest_version).await,
        InstallMethod::Rpm => update_rpm(&info.latest_version).await,
        InstallMethod::Binary => update_binary(&info.latest_version, &info.executable_path).await,
        InstallMethod::Snap => update_snap().await,
        InstallMethod::Unknown => {
            anyhow::bail!(
                "Could not detect installation method. Please update manually or use the install script:\n\
                curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh | bash"
            )
        }
    }
}

/// Helper function to download a file to temp directory
async fn download_to_temp(url: &str, filename: &str) -> Result<PathBuf> {
    println!("{}", format!("Downloading {}...", url).blue());
    
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to download file")?;
    
    if !response.status().is_success() {
        anyhow::bail!("Download failed with status: {}", response.status());
    }
    
    let bytes = response
        .bytes()
        .await
        .context("Failed to read response bytes")?;
    
    let temp_dir = env::temp_dir();
    let temp_path = temp_dir.join(filename);
    
    let mut file = fs::File::create(&temp_path)
        .context("Failed to create temp file")?;
    
    file.write_all(&bytes)
        .context("Failed to write to temp file")?;
    
    Ok(temp_path)
}

/// Get system OS
fn get_system_os() -> Result<String> {
    if cfg!(target_os = "linux") {
        Ok("linux".to_string())
    } else if cfg!(target_os = "macos") {
        Ok("macos".to_string())
    } else if cfg!(target_os = "windows") {
        Ok("windows".to_string())
    } else {
        Ok("unknown".to_string())
    }
}

/// Get system architecture
fn get_system_arch() -> Result<String> {
    if cfg!(target_arch = "x86_64") {
        Ok("x86_64".to_string())
    } else if cfg!(target_arch = "aarch64") {
        Ok("aarch64".to_string())
    } else if cfg!(target_arch = "arm") {
        Ok("armv7".to_string())
    } else if cfg!(target_arch = "x86") {
        Ok("i686".to_string())
    } else {
        Ok("unknown".to_string())
    }
}

/// Check if a file is writable
fn is_writable(path: &Path) -> Result<bool> {
    match fs::metadata(path) {
        Ok(_metadata) => {
            // Check if we can write to the parent directory
            if let Some(parent) = path.parent() {
                let test_file = parent.join(".rusty-commit-write-test");
                match fs::File::create(&test_file) {
                    Ok(_) => {
                        let _ = fs::remove_file(test_file);
                        Ok(true)
                    }
                    Err(_) => Ok(false),
                }
            } else {
                Ok(false)
            }
        }
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("1.0.1").unwrap();
        assert!(v2 > v1);
    }

    #[test]
    fn test_detect_install_method() {
        // This will vary based on how tests are run
        let method = detect_install_method();
        assert!(method.is_ok());
    }
}