// Build system for packaging games as standalone executables

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Build configuration
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Project name
    pub name: String,
    /// Project directory
    pub project_dir: PathBuf,
    /// Output directory for the built executable
    pub output_dir: PathBuf,
    /// Build profile (debug or release)
    pub profile: BuildProfile,
    /// Whether to package assets
    pub package_assets: bool,
    /// Whether to strip debug symbols
    pub strip_symbols: bool,
    /// Target platform (default: current platform)
    pub target: Option<String>,
}

/// Build profile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildProfile {
    Debug,
    Release,
}

impl BuildProfile {
    pub fn as_str(&self) -> &str {
        match self {
            BuildProfile::Debug => "debug",
            BuildProfile::Release => "release",
        }
    }
}

impl BuildConfig {
    /// Create a new build configuration
    pub fn new<P: AsRef<Path>>(name: String, project_dir: P) -> Self {
        let project_dir = project_dir.as_ref().to_path_buf();
        let output_dir = project_dir.join("build");

        Self {
            name,
            project_dir,
            output_dir,
            profile: BuildProfile::Release,
            package_assets: true,
            strip_symbols: true,
            target: None,
        }
    }

    /// Set build profile
    pub fn with_profile(mut self, profile: BuildProfile) -> Self {
        self.profile = profile;
        self
    }

    /// Set output directory
    pub fn with_output_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.output_dir = dir.as_ref().to_path_buf();
        self
    }

    /// Set whether to package assets
    pub fn with_package_assets(mut self, package: bool) -> Self {
        self.package_assets = package;
        self
    }

    /// Set whether to strip debug symbols
    pub fn with_strip_symbols(mut self, strip: bool) -> Self {
        self.strip_symbols = strip;
        self
    }

    /// Set target platform
    pub fn with_target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }
}

/// Game builder
pub struct GameBuilder {
    config: BuildConfig,
}

impl GameBuilder {
    /// Create a new game builder
    pub fn new(config: BuildConfig) -> Self {
        Self { config }
    }

    /// Build the game
    pub fn build(&self) -> Result<PathBuf> {
        log::info!("Building game: {}", self.config.name);
        log::info!("Profile: {:?}", self.config.profile);
        log::info!("Output: {:?}", self.config.output_dir);

        // Create output directory
        fs::create_dir_all(&self.config.output_dir)
            .context("Failed to create output directory")?;

        // Build the executable
        let exe_path = self.build_executable()?;

        // Package assets if enabled
        if self.config.package_assets {
            self.package_assets()?;
        }

        // Strip symbols if enabled
        if self.config.strip_symbols && self.config.profile == BuildProfile::Release {
            self.strip_symbols(&exe_path)?;
        }

        log::info!("Build complete: {:?}", exe_path);
        Ok(exe_path)
    }

    /// Build the executable using cargo
    fn build_executable(&self) -> Result<PathBuf> {
        log::info!("Compiling executable...");

        let mut cmd = Command::new("cargo");
        cmd.arg("build")
            .arg("--manifest-path")
            .arg(self.config.project_dir.join("Cargo.toml"))
            .current_dir(&self.config.project_dir);

        // Add release flag if needed
        if self.config.profile == BuildProfile::Release {
            cmd.arg("--release");
        }

        // Add target if specified
        if let Some(ref target) = self.config.target {
            cmd.arg("--target").arg(target);
        }

        // Execute build
        let output = cmd.output().context("Failed to execute cargo build")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Cargo build failed:\n{}", stderr);
        }

        // Find the built executable
        let target_dir = if let Some(ref target) = self.config.target {
            self.config.project_dir.join("target").join(target)
        } else {
            self.config.project_dir.join("target")
        };

        let exe_dir = target_dir.join(self.config.profile.as_str());
        let exe_name = if cfg!(windows) {
            format!("{}.exe", self.config.name)
        } else {
            self.config.name.clone()
        };

        let source_exe = exe_dir.join(&exe_name);
        let dest_exe = self.config.output_dir.join(&exe_name);

        // Copy executable to output directory
        fs::copy(&source_exe, &dest_exe)
            .with_context(|| format!("Failed to copy executable to {:?}", dest_exe))?;

        log::info!("Executable built: {:?}", dest_exe);
        Ok(dest_exe)
    }

    /// Package assets into output directory
    fn package_assets(&self) -> Result<()> {
        log::info!("Packaging assets...");

        let assets_src = self.config.project_dir.join("assets");
        let assets_dest = self.config.output_dir.join("assets");

        if !assets_src.exists() {
            log::warn!("Assets directory not found, skipping asset packaging");
            return Ok(());
        }

        // Copy assets directory recursively
        copy_dir_recursive(&assets_src, &assets_dest)
            .context("Failed to copy assets")?;

        log::info!("Assets packaged to: {:?}", assets_dest);
        Ok(())
    }

    /// Strip debug symbols from executable
    fn strip_symbols(&self, exe_path: &Path) -> Result<()> {
        log::info!("Stripping debug symbols...");

        let output = Command::new("strip")
            .arg(exe_path)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                log::info!("Debug symbols stripped");
                Ok(())
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::warn!("Strip command failed: {}", stderr);
                Ok(()) // Don't fail the build if strip fails
            }
            Err(_) => {
                log::warn!("Strip command not found, skipping symbol stripping");
                Ok(()) // Don't fail if strip is not available
            }
        }
    }

    /// Create a distributable package (zip/tar.gz)
    pub fn create_package(&self, _exe_path: &Path) -> Result<PathBuf> {
        log::info!("Creating distribution package...");

        let package_name = format!("{}-{}", self.config.name, self.get_platform_name());
        let package_path = if cfg!(windows) {
            self.config.output_dir.parent().unwrap().join(format!("{}.zip", package_name))
        } else {
            self.config.output_dir.parent().unwrap().join(format!("{}.tar.gz", package_name))
        };

        // Create archive based on platform
        #[cfg(unix)]
        self.create_tar_gz(&package_path)?;

        #[cfg(windows)]
        self.create_zip(&package_path)?;

        log::info!("Package created: {:?}", package_path);
        Ok(package_path)
    }

    #[cfg(unix)]
    fn create_tar_gz(&self, output_path: &Path) -> Result<()> {
        let output = Command::new("tar")
            .arg("czf")
            .arg(output_path)
            .arg("-C")
            .arg(self.config.output_dir.parent().unwrap())
            .arg(self.config.output_dir.file_name().unwrap())
            .output()
            .context("Failed to create tar.gz")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("tar command failed: {}", stderr);
        }

        Ok(())
    }

    #[cfg(windows)]
    fn create_zip(&self, output_path: &Path) -> Result<()> {
        // Use PowerShell's Compress-Archive on Windows
        let output = Command::new("powershell")
            .arg("-Command")
            .arg(format!(
                "Compress-Archive -Path '{}' -DestinationPath '{}'",
                self.config.output_dir.display(),
                output_path.display()
            ))
            .output()
            .context("Failed to create zip")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("PowerShell Compress-Archive failed: {}", stderr);
        }

        Ok(())
    }

    fn get_platform_name(&self) -> &str {
        if let Some(ref target) = self.config.target {
            target
        } else if cfg!(target_os = "windows") {
            "windows-x64"
        } else if cfg!(target_os = "macos") {
            "macos-x64"
        } else if cfg!(target_os = "linux") {
            "linux-x64"
        } else {
            "unknown"
        }
    }
}

/// Copy a directory recursively
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_config() {
        let config = BuildConfig::new("test_game".to_string(), "/tmp/test")
            .with_profile(BuildProfile::Release)
            .with_strip_symbols(true);

        assert_eq!(config.name, "test_game");
        assert_eq!(config.profile, BuildProfile::Release);
        assert!(config.strip_symbols);
    }

    #[test]
    fn test_build_profile() {
        assert_eq!(BuildProfile::Debug.as_str(), "debug");
        assert_eq!(BuildProfile::Release.as_str(), "release");
    }
}
