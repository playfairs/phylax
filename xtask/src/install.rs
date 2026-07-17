use anyhow::Result;
use std::process::Command;
use std::fs;
use std::path::Path;

pub fn run() -> Result<()> {
    println!("Installing Phylax...");

    build::run()?;

    #[cfg(target_os = "linux")]
    {
        install_linux()?;
    }

    #[cfg(target_os = "macos")]
    {
        install_macos()?;
    }

    #[cfg(target_os = "windows")]
    {
        install_windows()?;
    }

    println!("Installation completed");
    Ok(())
}

#[cfg(target_os = "linux")]
fn install_linux() -> Result<()> {
    let phylaxd_binary = "target/release/phylaxd";
    let phylaxctl_binary = "target/release/phylaxctl";

    if !Path::new(phylaxd_binary).exists() {
        anyhow::bail!("phylaxd binary not found. Run 'cargo xtask build' first.");
    }

    fs::create_dir_all("/usr/local/bin")?;
    fs::copy(phylaxd_binary, "/usr/local/bin/phylaxd")?;
    fs::copy(phylaxctl_binary, "/usr/local/bin/phylaxctl")?;

    let output = Command::new("chmod")
        .args(["+x", "/usr/local/bin/phylaxd", "/usr/local/bin/phylaxctl"])
        .status()?;

    if output.success() {
        println!("Installed to /usr/local/bin/");
        Ok(())
    } else {
        anyhow::bail!("Failed to set executable permissions")
    }
}

#[cfg(target_os = "macos")]
fn install_macos() -> Result<()> {
    let phylaxd_binary = "target/release/phylaxd";
    let phylaxctl_binary = "target/release/phylaxctl";

    if !Path::new(phylaxd_binary).exists() {
        anyhow::bail!("phylaxd binary not found. Run 'cargo xtask build' first.");
    }

    fs::create_dir_all("/usr/local/bin")?;
    fs::copy(phylaxd_binary, "/usr/local/bin/phylaxd")?;
    fs::copy(phylaxctl_binary, "/usr/local/bin/phylaxctl")?;

    let output = Command::new("chmod")
        .args(["+x", "/usr/local/bin/phylaxd", "/usr/local/bin/phylaxctl"])
        .status()?;

    if output.success() {
        println!("Installed to /usr/local/bin/");
        Ok(())
    } else {
        anyhow::bail!("Failed to set executable permissions")
    }
}

#[cfg(target_os = "windows")]
fn install_windows() -> Result<()> {
    let phylaxd_binary = "target/release/phylaxd.exe";
    let phylaxctl_binary = "target/release/phylaxctl.exe";

    if !Path::new(phylaxd_binary).exists() {
        anyhow::bail!("phylaxd.exe binary not found. Run 'cargo xtask build' first.");
    }

    let program_data = std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".to_string());
    let install_dir = format!("{}\\Phylax", program_data);
    fs::create_dir_all(&install_dir)?;

    fs::copy(phylaxd_binary, format!("{}\\phylaxd.exe", install_dir))?;
    fs::copy(phylaxctl_binary, format!("{}\\phylaxctl.exe", install_dir))?;

    println!("Installed to {}", install_dir);
    println!("Add {} to your PATH", install_dir);
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn install_linux() -> Result<()> {
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn install_macos() -> Result<()> {
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn install_windows() -> Result<()> {
    Ok(())
}