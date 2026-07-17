use anyhow::Result;
use std::process::Command;

pub fn run() -> Result<()> {
    println!("Building Phylax...");

    let status = Command::new("cargo")
        .args(["build", "--release"])
        .status()?;

    if status.success() {
        println!("Build completed successfully");
        Ok(())
    } else {
        anyhow::bail!("Build failed")
    }
}