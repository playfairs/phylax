use anyhow::Result;
use std::process::Command;

pub fn run() -> Result<()> {
    println!("Running tests...");

    let status = Command::new("cargo")
        .args(["test", "--release"])
        .status()?;

    if status.success() {
        println!("Tests passed");
        Ok(())
    } else {
        anyhow::bail!("Tests failed")
    }
}