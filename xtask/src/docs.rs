use anyhow::Result;
use std::process::Command;

pub fn run() -> Result<()> {
    println!("Generating documentation...");

    let status = Command::new("cargo")
        .args(["doc", "--no-deps", "--all-features"])
        .status()?;

    if status.success() {
        println!("Documentation generated at target/doc/");
        Ok(())
    } else {
        anyhow::bail!("Documentation generation failed")
    }
}