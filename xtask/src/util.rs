use anyhow::Result;
use std::process::Command;

pub fn run_command(command: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(command)
        .args(args)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Command '{}' failed", command)
    }
}

pub fn check_command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}