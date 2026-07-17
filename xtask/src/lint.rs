use anyhow::Result;
use std::process::Command;

pub fn run() -> Result<()> {
    println!("Running linters...");

    println!("Running cargo fmt --check...");
    let fmt_status = Command::new("cargo")
        .args(["fmt", "--check"])
        .status()?;

    if !fmt_status.success() {
        anyhow::bail!("Formatting check failed. Run 'cargo fmt' to fix.");
    }

    println!("Running cargo clippy...");
    let clippy_status = Command::new("cargo")
        .args(["clippy", "--all-targets", "--", "-D", "warnings"])
        .status()?;

    if !clippy_status.success() {
        anyhow::bail!("Clippy failed");
    }

    println!("Linting passed");
    Ok(())
}