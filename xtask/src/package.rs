use anyhow::Result;
use std::process::Command;
use std::fs;

pub fn run() -> Result<()> {
    println!("Packaging Phylax...");

    build::run()?;

    let package_dir = "target/package";
    fs::create_dir_all(package_dir)?;

    #[cfg(target_os = "linux")]
    {
        println!("Creating Linux package...");
        create_linux_package()?;
    }

    #[cfg(target_os = "macos")]
    {
        println!("Creating macOS package...");
        create_macos_package()?;
    }

    #[cfg(target_os = "windows")]
    {
        println!("Creating Windows package...");
        create_windows_package()?;
    }

    println!("Packaging completed");
    Ok(())
}

#[cfg(target_os = "linux")]
fn create_linux_package() -> Result<()> {
    use std::path::Path;

    let phylaxd_binary = "target/release/phylaxd";
    let phylaxctl_binary = "target/release/phylaxctl";

    if !Path::new(phylaxd_binary).exists() {
        anyhow::bail!("phylaxd binary not found. Run 'cargo xtask build' first.");
    }

    let output = Command::new("tar")
        .args(["-czf", "target/package/phylax-linux.tar.gz", "-C", "target/release", "phylaxd", "phylaxctl"])
        .status()?;

    if output.success() {
        println!("Linux package created: target/package/phylax-linux.tar.gz");
        Ok(())
    } else {
        anyhow::bail!("Failed to create Linux package")
    }
}

#[cfg(target_os = "macos")]
fn create_macos_package() -> Result<()> {
    use std::path::Path;

    let phylaxd_binary = "target/release/phylaxd";
    let phylaxctl_binary = "target/release/phylaxctl";

    if !Path::new(phylaxd_binary).exists() {
        anyhow::bail!("phylaxd binary not found. Run 'cargo xtask build' first.");
    }

    let output = Command::new("tar")
        .args(["-czf", "target/package/phylax-macos.tar.gz", "-C", "target/release", "phylaxd", "phylaxctl"])
        .status()?;

    if output.success() {
        println!("macOS package created: target/package/phylax-macos.tar.gz");
        Ok(())
    } else {
        anyhow::bail!("Failed to create macOS package")
    }
}

#[cfg(target_os = "windows")]
fn create_windows_package() -> Result<()> {
    use std::path::Path;

    let phylaxd_binary = "target/release/phylaxd.exe";
    let phylaxctl_binary = "target/release/phylaxctl.exe";

    if !Path::new(phylaxd_binary).exists() {
        anyhow::bail!("phylaxd binary not found. Run 'cargo xtask build' first.");
    }

    let output = Command::new("powershell")
        .args(["-Command", "Compress-Archive -Path target/release/phylaxd.exe,target/release/phylaxctl.exe -DestinationPath target/package/phylax-windows.zip"])
        .status()?;

    if output.success() {
        println!("Windows package created: target/package/phylax-windows.zip");
        Ok(())
    } else {
        anyhow::bail!("Failed to create Windows package")
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn create_linux_package() -> Result<()> {
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn create_macos_package() -> Result<()> {
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn create_windows_package() -> Result<()> {
    Ok(())
}