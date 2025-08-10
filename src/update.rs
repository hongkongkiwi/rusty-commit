use anyhow::{bail, Context, Result};
use colored::*;
use reqwest;
use semver::Version;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const GITHUB_REPO: &str = "hongkongkiwi/rusty-commit";
const MAX_DOWNLOAD_SIZE: u64 = 100 * 1024 * 1024; // 100MB max
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

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

/// Validate version string format
fn validate_version(version: &str) -> Result<()> {
    let clean_version = version.trim_start_matches('v');
    Version::parse(clean_version)
        .context("Invalid version format")?;
    
    // Additional validation: no path traversal
    if version.contains("..") || version.contains('/') || version.contains('\\') {
        bail!("Invalid characters in version string");
    }
    
    Ok(())
}

/// Validate and sanitize file paths
fn sanitize_path(path: &Path) -> Result<PathBuf> {
    // Resolve to absolute path and check for path traversal
    let canonical = path
        .canonicalize()
        .context("Failed to resolve path")?;
    
    // Ensure path doesn't escape expected directories
    let path_str = canonical.to_string_lossy();
    if path_str.contains("..") {
        bail!("Path traversal detected");
    }
    
    Ok(canonical)
}

/// Create a secure HTTP client with proper timeouts and limits
fn create_http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(format!("rusty-commit/{}", env!("CARGO_PKG_VERSION")))
        .timeout(DOWNLOAD_TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .https_only(true)
        .build()
        .context("Failed to create HTTP client")
}

/// Detect how rusty-commit was installed (more secure version)
pub fn detect_install_method() -> Result<InstallMethod> {
    let exe_path = env::current_exe()
        .context("Failed to get current executable path")?;
    
    // Sanitize the path
    let exe_path = sanitize_path(&exe_path)?;
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
        // Try to detect package manager using safer methods
        if Path::new("/etc/debian_version").exists() {
            // Use dpkg-query which is safer than dpkg -S
            if let Ok(output) = Command::new("dpkg-query")
                .args(&["-S", &exe_path.to_string_lossy()])
                .output()
            {
                if output.status.success() {
                    return Ok(InstallMethod::Deb);
                }
            }
        }
        
        if Path::new("/etc/redhat-release").exists() || Path::new("/etc/fedora-release").exists() {
            // Check if installed via rpm (safer query)
            if let Ok(output) = Command::new("rpm")
                .args(&["-qf", &exe_path.to_string_lossy()])
                .output()
            {
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

/// Get the latest version from GitHub releases (with validation)
pub async fn get_latest_version() -> Result<String> {
    let client = create_http_client()?;
    
    let url = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to fetch latest release")?;
    
    if !response.status().is_success() {
        bail!("GitHub API returned status: {}", response.status());
    }
    
    let release: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse release JSON")?;
    
    let tag_name = release["tag_name"]
        .as_str()
        .context("Failed to get tag_name from release")?;
    
    // Validate version format
    validate_version(tag_name)?;
    
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

/// Update using Homebrew (safer version)
async fn update_homebrew() -> Result<()> {
    println!("{}", "Updating via Homebrew...".blue());
    
    // Check if brew exists first
    which::which("brew")
        .context("Homebrew not found in PATH")?;
    
    // Update Homebrew
    let output = Command::new("brew")
        .args(&["update"])
        .output()
        .context("Failed to run brew update")?;
    
    if !output.status.success() {
        bail!("brew update failed: {}", String::from_utf8_lossy(&output.stderr));
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
        bail!("brew upgrade failed: {}", stderr);
    }
    
    println!("{}", "Successfully updated via Homebrew!".green());
    Ok(())
}

/// Update using Cargo (safer version)
async fn update_cargo() -> Result<()> {
    println!("{}", "Updating via Cargo...".blue());
    
    // Check if cargo exists
    which::which("cargo")
        .context("Cargo not found in PATH")?;
    
    let output = Command::new("cargo")
        .args(&["install", "rusty-commit", "--force", "--features", "secure-storage"])
        .output()
        .context("Failed to run cargo install")?;
    
    if !output.status.success() {
        bail!("cargo install failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    println!("{}", "Successfully updated via Cargo!".green());
    Ok(())
}

/// Download with size limit and checksum verification
async fn download_with_verification(
    url: &str,
    expected_checksum: Option<&str>,
    max_size: u64,
) -> Result<Vec<u8>> {
    println!("{}", format!("Downloading from: {}", url).blue());
    
    let client = create_http_client()?;
    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to start download")?;
    
    if !response.status().is_success() {
        bail!("Download failed with status: {}", response.status());
    }
    
    // Check content length if available
    if let Some(content_length) = response.content_length() {
        if content_length > max_size {
            bail!("File too large: {} bytes (max: {} bytes)", content_length, max_size);
        }
    }
    
    // Download with size limit
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    use futures::StreamExt;
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Failed to read chunk")?;
        bytes.extend_from_slice(&chunk);
        
        if bytes.len() as u64 > max_size {
            bail!("Download exceeded maximum size of {} bytes", max_size);
        }
    }
    
    // Verify checksum if provided
    if let Some(expected) = expected_checksum {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let actual = format!("{:x}", hasher.finalize());
        
        if actual != expected {
            bail!("Checksum verification failed");
        }
        
        println!("{}", "Checksum verified".green());
    }
    
    // TODO: Add Cosign signature verification in future version
    // This would require integrating with cosign binary or sigstore-rs crate
    // For now, checksums provide integrity verification
    
    Ok(bytes)
}

/// Get SHA256 checksum for a release file
async fn get_release_checksum(version: &str, filename: &str) -> Result<Option<String>> {
    let client = create_http_client()?;
    let url = format!(
        "https://github.com/{}/releases/download/v{}/SHA256SUMS.txt",
        GITHUB_REPO, version
    );
    
    let response = client.get(&url).send().await;
    
    match response {
        Ok(resp) if resp.status().is_success() => {
            let text = resp.text().await?;
            for line in text.lines() {
                if line.contains(filename) {
                    if let Some(checksum) = line.split_whitespace().next() {
                        return Ok(Some(checksum.to_string()));
                    }
                }
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

/// Atomic file replacement with proper error handling
async fn atomic_replace_file(source: &Path, target: &Path) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::copy;
    
    // Create a unique temporary file in the same directory as target
    let temp_path = target.with_extension(format!(".tmp.{}", std::process::id()));
    
    // Copy source to temp location
    {
        let mut source_file = fs::File::open(source)
            .context("Failed to open source file")?;
        let mut temp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)
            .context("Failed to create temp file")?;
        
        copy(&mut source_file, &mut temp_file)
            .context("Failed to copy to temp file")?;
    }
    
    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_path, perms)?;
    }
    
    // Atomic rename
    fs::rename(&temp_path, target)
        .context("Failed to perform atomic rename")?;
    
    Ok(())
}

/// Update Debian package (secure version)
async fn update_deb(version: &str) -> Result<()> {
    println!("{}", "Updating via apt/dpkg...".blue());
    
    // Validate version
    validate_version(version)?;
    
    let arch = get_system_arch()?;
    let deb_arch = match arch.as_str() {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "armv7" => "armhf",
        _ => bail!("Unsupported architecture for .deb: {}", arch),
    };
    
    let filename = format!("rusty-commit_{}_{}.deb", version, deb_arch);
    let url = format!(
        "https://github.com/{}/releases/download/v{}/{}",
        GITHUB_REPO, version, filename
    );
    
    // Get checksum
    let checksum = get_release_checksum(version, &filename).await?;
    
    // Download with verification
    let package_data = download_with_verification(&url, checksum.as_deref(), MAX_DOWNLOAD_SIZE).await?;
    
    // Save to secure temp directory
    let temp_dir = tempfile::TempDir::new()?;
    let temp_path = temp_dir.path().join(&filename);
    fs::write(&temp_path, package_data)?;
    
    // Install with dpkg or apt
    let result = if which::which("apt-get").is_ok() {
        Command::new("sudo")
            .args(&["apt-get", "install", "-y"])
            .arg(&temp_path)
            .output()
    } else if which::which("dpkg").is_ok() {
        Command::new("sudo")
            .args(&["dpkg", "-i"])
            .arg(&temp_path)
            .output()
    } else {
        bail!("Neither apt-get nor dpkg found");
    };
    
    match result {
        Ok(output) if output.status.success() => {
            println!("{}", "Successfully updated via package manager!".green());
            Ok(())
        }
        Ok(output) => bail!("Package installation failed: {}", String::from_utf8_lossy(&output.stderr)),
        Err(e) => Err(e.into()),
    }
}

/// Update RPM package (secure version)
async fn update_rpm(version: &str) -> Result<()> {
    println!("{}", "Updating via rpm/dnf/yum...".blue());
    
    // Validate version
    validate_version(version)?;
    
    let arch = get_system_arch()?;
    let rpm_arch = match arch.as_str() {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => bail!("Unsupported architecture for .rpm: {}", arch),
    };
    
    let filename = format!("rusty-commit-{}-1.{}.rpm", version, rpm_arch);
    let url = format!(
        "https://github.com/{}/releases/download/v{}/{}",
        GITHUB_REPO, version, filename
    );
    
    // Get checksum
    let checksum = get_release_checksum(version, &filename).await?;
    
    // Download with verification
    let package_data = download_with_verification(&url, checksum.as_deref(), MAX_DOWNLOAD_SIZE).await?;
    
    // Save to secure temp directory
    let temp_dir = tempfile::TempDir::new()?;
    let temp_path = temp_dir.path().join(&filename);
    fs::write(&temp_path, package_data)?;
    
    // Install with package manager
    let result = if which::which("dnf").is_ok() {
        Command::new("sudo")
            .args(&["dnf", "install", "-y"])
            .arg(&temp_path)
            .output()
    } else if which::which("yum").is_ok() {
        Command::new("sudo")
            .args(&["yum", "install", "-y"])
            .arg(&temp_path)
            .output()
    } else if which::which("rpm").is_ok() {
        Command::new("sudo")
            .args(&["rpm", "-Uvh"])
            .arg(&temp_path)
            .output()
    } else {
        bail!("No suitable package manager found");
    };
    
    match result {
        Ok(output) if output.status.success() => {
            println!("{}", "Successfully updated via package manager!".green());
            Ok(())
        }
        Ok(output) => bail!("Package installation failed: {}", String::from_utf8_lossy(&output.stderr)),
        Err(e) => Err(e.into()),
    }
}

/// Update binary installation (secure version)
async fn update_binary(version: &str, exe_path: &Path) -> Result<()> {
    println!("{}", "Updating binary installation...".blue());
    
    // Validate inputs
    validate_version(version)?;
    let exe_path = sanitize_path(exe_path)?;
    
    let os = get_system_os()?;
    let arch = get_system_arch()?;
    
    // Prefer musl tarballs when running on Alpine/musl
    let is_musl = if os == "linux" {
        // Best-effort detection: check /etc/alpine-release or ldd output
        if Path::new("/etc/alpine-release").exists() {
            true
        } else {
            let output = Command::new("sh").arg("-lc").arg("ldd --version 2>&1 || true").output();
            if let Ok(out) = output {
                String::from_utf8_lossy(&out.stdout).to_lowercase().contains("musl") ||
                String::from_utf8_lossy(&out.stderr).to_lowercase().contains("musl")
            } else { false }
        }
    } else { false };

    let archive_name = match (os.as_str(), arch.as_str(), is_musl) {
        ("linux", "x86_64", true) => "rustycommit-linux-musl-x86_64.tar.gz",
        ("linux", "aarch64", true) => "rustycommit-linux-musl-aarch64.tar.gz",
        ("linux", "riscv64", true) => "rustycommit-linux-musl-riscv64.tar.gz",
        ("linux", "x86_64", false) => "rustycommit-linux-x86_64.tar.gz",
        ("linux", "aarch64", false) => "rustycommit-linux-aarch64.tar.gz",
        ("linux", "armv7", false) => "rustycommit-linux-armv7.tar.gz",
        ("linux", "riscv64", false) => "rustycommit-linux-riscv64.tar.gz",
        ("macos", "x86_64", _) => "rustycommit-macos-x86_64.tar.gz",
        ("macos", "aarch64", _) => "rustycommit-macos-aarch64.tar.gz",
        ("windows", "x86_64", _) => "rustycommit-windows-x86_64.zip",
        ("windows", "i686", _) => "rustycommit-windows-i686.zip",
        _ => bail!("Unsupported OS/architecture: {}-{} (musl={})", os, arch, is_musl),
    };
    
    let url = format!(
        "https://github.com/{}/releases/download/v{}/{}",
        GITHUB_REPO, version, archive_name
    );
    
    // Get checksum
    let checksum = get_release_checksum(version, archive_name).await?;
    
    // Download with verification
    let archive_data = download_with_verification(&url, checksum.as_deref(), MAX_DOWNLOAD_SIZE).await?;
    
    // Extract to secure temp directory
    let temp_dir = tempfile::TempDir::new()?;
    let archive_path = temp_dir.path().join(archive_name);
    fs::write(&archive_path, archive_data)?;
    
    // Extract archive using built-in libraries when possible
    let binary_name = if cfg!(windows) { "rco.exe" } else { "rco" };
    let extracted_binary = temp_dir.path().join(binary_name);
    
    if archive_name.ends_with(".tar.gz") {
        // Use tar crate for extraction (safer than shell command)
        use flate2::read::GzDecoder;
        use tar::Archive;
        
        let tar_gz = fs::File::open(&archive_path)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        archive.unpack(temp_dir.path())?;
    } else if archive_name.ends_with(".zip") {
        // Use zip crate for extraction
        use zip::ZipArchive;
        
        let file = fs::File::open(&archive_path)?;
        let mut archive = ZipArchive::new(file)?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name() == binary_name {
                let mut outfile = fs::File::create(&extracted_binary)?;
                io::copy(&mut file, &mut outfile)?;
                break;
            }
        }
    }
    
    if !extracted_binary.exists() {
        bail!("Binary not found in archive");
    }
    
    // Create backup of current binary
    let backup_path = exe_path.with_extension(format!("bak.{}", std::process::id()));
    fs::copy(&exe_path, &backup_path)
        .context("Failed to create backup")?;
    
    // Try to perform atomic replacement
    let replace_result = atomic_replace_file(&extracted_binary, &exe_path).await;
    
    match replace_result {
        Ok(_) => {
            // Success - remove backup
            let _ = fs::remove_file(&backup_path);
            println!("{}", "Successfully updated binary!".green());
            Ok(())
        }
        Err(e) => {
            // Try to restore backup
            if let Err(restore_err) = fs::rename(&backup_path, &exe_path) {
                eprintln!("{}", format!("Critical: Failed to restore backup: {}", restore_err).red());
            }
            Err(e)
        }
    }
}

/// Update Snap package
async fn update_snap() -> Result<()> {
    println!("{}", "Updating via Snap...".blue());
    
    which::which("snap")
        .context("Snap not found in PATH")?;
    
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
        bail!("snap refresh failed: {}", stderr);
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
            bail!(
                "Could not detect installation method. Please update manually or use the install script:\n\
                curl -fsSL https://raw.githubusercontent.com/{}/main/install.sh | bash",
                GITHUB_REPO
            )
        }
    }
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
    } else if cfg!(target_arch = "riscv64") {
        Ok("riscv64".to_string())
    } else {
        Ok("unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_validation() {
        assert!(validate_version("1.0.0").is_ok());
        assert!(validate_version("v1.0.0").is_ok());
        assert!(validate_version("1.0.0-beta.1").is_ok());
        
        assert!(validate_version("../etc/passwd").is_err());
        assert!(validate_version("1.0.0/../../etc").is_err());
        assert!(validate_version("invalid").is_err());
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("1.0.1").unwrap();
        assert!(v2 > v1);
    }
}