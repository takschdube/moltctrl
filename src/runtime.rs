//! Auto-downloads Node.js and OpenClaw on first use.
//!
//! Users never need to manually install Node.js or OpenClaw — moltctrl handles it.
//!
//! Directory layout under `~/.moltctrl/`:
//! ```text
//! ~/.moltctrl/
//! ├── runtime/
//! │   └── node/          # Portable Node.js binary
//! │       └── bin/
//! │           └── node
//! ├── openclaw/          # OpenClaw installation
//! │   └── node_modules/
//! │       └── openclaw/
//! └── instances/
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};

/// The Node.js version we download when the system doesn't have >=22.
const NODE_VERSION: &str = "22.14.0";

/// Minimum acceptable system Node.js major version.
const NODE_MIN_MAJOR: u32 = 22;

// ---------------------------------------------------------------------------
// Download URLs by (os, arch)
// ---------------------------------------------------------------------------

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const NODE_DOWNLOAD_URL: &str = "https://nodejs.org/dist/v22.14.0/node-v22.14.0-linux-x64.tar.xz";
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const NODE_ARCHIVE_NAME: &str = "node-v22.14.0-linux-x64";
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const NODE_ARCHIVE_EXT: &str = "tar.xz";

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
const NODE_DOWNLOAD_URL: &str = "https://nodejs.org/dist/v22.14.0/node-v22.14.0-linux-arm64.tar.xz";
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
const NODE_ARCHIVE_NAME: &str = "node-v22.14.0-linux-arm64";
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
const NODE_ARCHIVE_EXT: &str = "tar.xz";

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const NODE_DOWNLOAD_URL: &str = "https://nodejs.org/dist/v22.14.0/node-v22.14.0-darwin-x64.tar.gz";
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const NODE_ARCHIVE_NAME: &str = "node-v22.14.0-darwin-x64";
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const NODE_ARCHIVE_EXT: &str = "tar.gz";

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const NODE_DOWNLOAD_URL: &str =
    "https://nodejs.org/dist/v22.14.0/node-v22.14.0-darwin-arm64.tar.gz";
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const NODE_ARCHIVE_NAME: &str = "node-v22.14.0-darwin-arm64";
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const NODE_ARCHIVE_EXT: &str = "tar.gz";

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const NODE_DOWNLOAD_URL: &str = "https://nodejs.org/dist/v22.14.0/node-v22.14.0-win-x64.zip";
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const NODE_ARCHIVE_NAME: &str = "node-v22.14.0-win-x64";
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const NODE_ARCHIVE_EXT: &str = "zip";

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Main entry point. Ensures Node.js >= 22 and OpenClaw are available.
///
/// Returns the path to the directory containing the `openclaw` package
/// (i.e. `~/.moltctrl/openclaw/`), which can be used with [`openclaw_command`].
pub fn ensure_runtime() -> Result<PathBuf> {
    let base_dir = moltctrl_home()?;

    // Step 1 — Resolve a usable Node.js binary.
    let node_path = match get_system_node() {
        Some(p) => {
            eprintln!(
                "  Using system Node.js ({})",
                node_version_string(&p).unwrap_or_else(|| "unknown".into())
            );
            p
        }
        None => {
            let runtime_dir = base_dir.join("runtime").join("node");
            let portable_node = node_binary_path(&runtime_dir);
            if !portable_node.exists() {
                eprintln!(
                    "  No system Node.js >= {NODE_MIN_MAJOR} found. Downloading portable runtime..."
                );
                download_node(&runtime_dir).context("Failed to download portable Node.js")?;
            }
            if !portable_node.exists() {
                bail!(
                    "Portable Node.js binary not found at {} after download",
                    portable_node.display()
                );
            }
            eprintln!("  Using portable Node.js v{NODE_VERSION}");
            portable_node
        }
    };

    // Step 2 — Ensure OpenClaw is installed.
    let openclaw_dir = base_dir.join("openclaw");
    let openclaw_pkg = openclaw_dir.join("node_modules").join("openclaw");
    if !openclaw_pkg.exists() {
        eprintln!("  Installing OpenClaw...");
        install_openclaw(&node_path, &openclaw_dir).context("Failed to install OpenClaw")?;
    }

    if !openclaw_pkg.exists() {
        bail!(
            "OpenClaw package not found at {} after installation",
            openclaw_pkg.display()
        );
    }

    Ok(openclaw_dir)
}

/// Returns `(program, args)` needed to invoke the OpenClaw CLI.
///
/// Checks, in order:
/// 1. A system-wide `openclaw` on `$PATH`.
/// 2. The managed installation at `~/.moltctrl/openclaw/`.
pub fn openclaw_command() -> Result<(String, Vec<String>)> {
    // 1. System openclaw?
    if let Some(system_oc) = which_openclaw() {
        return Ok((system_oc.to_string_lossy().into_owned(), Vec::new()));
    }

    // 2. Managed installation.
    let base_dir = moltctrl_home()?;
    let openclaw_dir = base_dir.join("openclaw");
    let openclaw_bin = openclaw_bin_path(&openclaw_dir);

    if !openclaw_bin.exists() {
        bail!("OpenClaw not found. Run `ensure_runtime()` first, or install openclaw globally.");
    }

    // We need to figure out which node to invoke it with.
    let node = match get_system_node() {
        Some(p) => p,
        None => {
            let runtime_dir = base_dir.join("runtime").join("node");
            let p = node_binary_path(&runtime_dir);
            if !p.exists() {
                bail!("No Node.js available to run OpenClaw. Run `ensure_runtime()` first.");
            }
            p
        }
    };

    Ok((
        node.to_string_lossy().into_owned(),
        vec![openclaw_bin.to_string_lossy().into_owned()],
    ))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Returns `~/.moltctrl`, creating it if necessary.
fn moltctrl_home() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let base = home.join(".moltctrl");
    std::fs::create_dir_all(&base)
        .with_context(|| format!("Failed to create {}", base.display()))?;
    Ok(base)
}

/// Check for a system Node.js >= [`NODE_MIN_MAJOR`] on `$PATH`.
fn get_system_node() -> Option<PathBuf> {
    let output = Command::new("node").arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let version_str = String::from_utf8_lossy(&output.stdout);
    let major = parse_node_major(version_str.trim())?;
    if major >= NODE_MIN_MAJOR {
        resolve_binary("node")
    } else {
        None
    }
}

/// Parse a version string like `v22.14.0` and return the major version.
fn parse_node_major(version: &str) -> Option<u32> {
    let version = version.strip_prefix('v').unwrap_or(version);
    let major_str = version.split('.').next()?;
    major_str.parse::<u32>().ok()
}

/// Get the version string for a given node binary path.
fn node_version_string(node: &Path) -> Option<String> {
    let output = Command::new(node).arg("--version").output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Download and extract a portable Node.js distribution into `dest`.
fn download_node(dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)
        .with_context(|| format!("Failed to create directory {}", dest.display()))?;

    let archive_file = dest.join(format!("node-download.{NODE_ARCHIVE_EXT}"));

    // Download with progress bar.
    download_file(
        NODE_DOWNLOAD_URL,
        &archive_file,
        &format!("Downloading Node.js v{NODE_VERSION}"),
    )?;

    // Extract.
    extract_archive(&archive_file, dest)?;

    // The archive extracts to a subdirectory like `node-v22.14.0-linux-x64/`.
    // Move contents up so that `dest/bin/node` exists (Unix) or `dest/node.exe`
    // (Windows).
    let extracted_dir = dest.join(NODE_ARCHIVE_NAME);
    if extracted_dir.exists() {
        move_dir_contents(&extracted_dir, dest)?;
        let _ = std::fs::remove_dir_all(&extracted_dir);
    }

    // Clean up archive file.
    let _ = std::fs::remove_file(&archive_file);

    Ok(())
}

/// Install the `openclaw` npm package into `dest`.
fn install_openclaw(node_path: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)
        .with_context(|| format!("Failed to create directory {}", dest.display()))?;

    // We need npm, which lives next to the node binary.
    let npm_path = npm_path_from_node(node_path);
    if !npm_path.exists() {
        bail!("npm not found at {}", npm_path.display());
    }

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("  {spinner:.cyan} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
    );
    spinner.set_message("Installing OpenClaw via npm...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    // Create a minimal package.json so npm install works in this directory.
    let pkg_json = dest.join("package.json");
    if !pkg_json.exists() {
        std::fs::write(
            &pkg_json,
            r#"{"name":"moltctrl-openclaw","version":"1.0.0","private":true}"#,
        )?;
    }

    // Run: node <npm-cli.js> install openclaw
    let output = Command::new(node_path.as_os_str())
        .arg(npm_path.as_os_str())
        .arg("install")
        .arg("openclaw")
        .current_dir(dest)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .context("Failed to run npm install")?;

    spinner.finish_and_clear();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        bail!(
            "npm install openclaw failed (exit {}):\nstdout: {}\nstderr: {}",
            output.status,
            stdout,
            stderr
        );
    }

    eprintln!("  OpenClaw installed successfully.");
    Ok(())
}

/// Download a file from `url` to `dest` with a progress bar.
fn download_file(url: &str, dest: &Path, label: &str) -> Result<()> {
    use std::io::Write;

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .context("Failed to build HTTP client")?;

    let response = client
        .get(url)
        .send()
        .with_context(|| format!("Failed to GET {url}"))?;

    if !response.status().is_success() {
        bail!("HTTP {} downloading {}", response.status(), url);
    }

    let total_size = response.content_length().unwrap_or(0);

    let pb = if total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::with_template(
                "  {msg}\n  [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
            )
            .unwrap()
            .progress_chars("##-"),
        );
        pb.set_message(label.to_string());
        pb
    } else {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::with_template("  {spinner:.cyan} {msg} ({bytes})")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
        );
        pb.set_message(label.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        pb
    };

    let mut file = std::fs::File::create(dest)
        .with_context(|| format!("Failed to create {}", dest.display()))?;

    let mut downloaded: u64 = 0;
    let mut reader = response;

    loop {
        let mut buf = [0u8; 8192];
        let n =
            std::io::Read::read(&mut reader, &mut buf).context("Error reading download stream")?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).context("Error writing to file")?;
        downloaded += n as u64;
        pb.set_position(downloaded);
    }

    pb.finish_and_clear();
    eprintln!(
        "  Downloaded {} ({:.1} MB)",
        dest.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        downloaded as f64 / 1_048_576.0
    );

    Ok(())
}

/// Extract an archive (tar.xz, tar.gz, or zip) into `dest`.
fn extract_archive(archive: &Path, dest: &Path) -> Result<()> {
    let archive_str = archive
        .to_str()
        .context("Archive path is not valid UTF-8")?;

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("  {spinner:.cyan} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
    );
    spinner.set_message("Extracting archive...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    #[cfg(unix)]
    {
        let tar_flags = if archive_str.ends_with(".tar.xz") {
            "xJf"
        } else if archive_str.ends_with(".tar.gz") {
            "xzf"
        } else {
            bail!("Unsupported archive format: {archive_str}");
        };

        let status = Command::new("tar")
            .arg(tar_flags)
            .arg(archive_str)
            .arg("-C")
            .arg(dest.as_os_str())
            .status()
            .context("Failed to run tar")?;

        if !status.success() {
            bail!("tar extraction failed with exit code {status}");
        }
    }

    #[cfg(windows)]
    {
        if archive_str.ends_with(".zip") {
            let dest_str = dest.to_str().context("Dest path is not valid UTF-8")?;
            let status = Command::new("powershell")
                .arg("-NoProfile")
                .arg("-Command")
                .arg(format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    archive_str, dest_str
                ))
                .status()
                .context("Failed to run PowerShell Expand-Archive")?;
            if !status.success() {
                bail!("Expand-Archive failed with exit code {status}");
            }
        } else {
            bail!("Unsupported archive format on Windows: {archive_str}");
        }
    }

    spinner.finish_and_clear();
    eprintln!("  Extraction complete.");
    Ok(())
}

/// Move all entries from `src_dir` into `dest_dir`.
fn move_dir_contents(src_dir: &Path, dest_dir: &Path) -> Result<()> {
    let entries = std::fs::read_dir(src_dir)
        .with_context(|| format!("Failed to read directory {}", src_dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let src_path = entry.path();
        let dest_path = dest_dir.join(&file_name);

        if src_path == dest_path {
            continue;
        }

        if dest_path.exists() {
            if dest_path.is_dir() {
                std::fs::remove_dir_all(&dest_path)?;
            } else {
                std::fs::remove_file(&dest_path)?;
            }
        }

        std::fs::rename(&src_path, &dest_path).with_context(|| {
            format!(
                "Failed to move {} -> {}",
                src_path.display(),
                dest_path.display()
            )
        })?;
    }

    Ok(())
}

/// Resolve the full path to a binary on `$PATH`.
fn resolve_binary(name: &str) -> Option<PathBuf> {
    #[cfg(unix)]
    let cmd = "which";
    #[cfg(windows)]
    let cmd = "where";

    let output = Command::new(cmd).arg(name).output().ok()?;
    if output.status.success() {
        let path_str = String::from_utf8_lossy(&output.stdout)
            .trim()
            .lines()
            .next()?
            .to_string();
        Some(PathBuf::from(path_str))
    } else {
        None
    }
}

/// Check if `openclaw` is on the system `$PATH`.
fn which_openclaw() -> Option<PathBuf> {
    let bin = resolve_binary("openclaw")?;
    if bin.exists() {
        Some(bin)
    } else {
        None
    }
}

/// Path to the node binary inside a portable installation directory.
fn node_binary_path(runtime_dir: &Path) -> PathBuf {
    #[cfg(unix)]
    {
        runtime_dir.join("bin").join("node")
    }
    #[cfg(windows)]
    {
        runtime_dir.join("node.exe")
    }
}

/// Given a path to the `node` binary, derive the path to the `npm` CLI script.
fn npm_path_from_node(node_path: &Path) -> PathBuf {
    let bin_dir = node_path
        .parent()
        .expect("node binary should have a parent directory");

    #[cfg(unix)]
    {
        // npm cli entry point: ../lib/node_modules/npm/bin/npm-cli.js
        let npm_cli = bin_dir
            .parent()
            .map(|p| {
                p.join("lib")
                    .join("node_modules")
                    .join("npm")
                    .join("bin")
                    .join("npm-cli.js")
            })
            .unwrap_or_else(|| bin_dir.join("npm"));
        if npm_cli.exists() {
            npm_cli
        } else {
            bin_dir.join("npm")
        }
    }

    #[cfg(windows)]
    {
        let npm_cli = bin_dir
            .join("node_modules")
            .join("npm")
            .join("bin")
            .join("npm-cli.js");
        if npm_cli.exists() {
            npm_cli
        } else {
            bin_dir.join("npm.cmd")
        }
    }
}

/// Path to the OpenClaw binary/script inside the managed install directory.
fn openclaw_bin_path(openclaw_dir: &Path) -> PathBuf {
    #[cfg(unix)]
    {
        openclaw_dir
            .join("node_modules")
            .join(".bin")
            .join("openclaw")
    }
    #[cfg(windows)]
    {
        openclaw_dir
            .join("node_modules")
            .join(".bin")
            .join("openclaw.cmd")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_node_major() {
        assert_eq!(parse_node_major("v22.14.0"), Some(22));
        assert_eq!(parse_node_major("v20.11.1"), Some(20));
        assert_eq!(parse_node_major("v18.0.0"), Some(18));
        assert_eq!(parse_node_major("22.14.0"), Some(22));
        assert_eq!(parse_node_major(""), None);
    }

    #[test]
    fn test_node_binary_path() {
        let dir = PathBuf::from("/tmp/test-runtime/node");
        let bin = node_binary_path(&dir);
        #[cfg(unix)]
        assert_eq!(bin, PathBuf::from("/tmp/test-runtime/node/bin/node"));
        #[cfg(windows)]
        assert_eq!(bin, PathBuf::from("/tmp/test-runtime/node/node.exe"));
    }

    #[test]
    fn test_moltctrl_home() {
        let home = moltctrl_home().unwrap();
        assert!(home.ends_with(".moltctrl"));
        assert!(home.exists());
    }
}
