use phylax_common::{Error, Result, get_data_path};
use phylax_storage::Storage;
use phylax_events::Event;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tabled::{Table, Tabled};
use colored::Colorize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "/var/lib/phylax/phylax.db")]
    database: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Status,
    Logs {
        #[arg(short, long, default_value = "50")]
        lines: usize,
    },
    Events {
        #[arg(short, long)]
        event_type: Option<String>,
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    Incidents {
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    Rules,
    Reload,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let db_path = PathBuf::from(args.database);
    let storage = Storage::new(&db_path)?;

    match args.command {
        Commands::Status => show_status(&storage)?,
        Commands::Logs { lines } => show_logs(&storage, lines)?,
        Commands::Events { event_type, limit } => show_events(&storage, event_type, limit)?,
        Commands::Incidents { limit } => show_incidents(&storage, limit)?,
        Commands::Rules => show_rules(&storage)?,
        Commands::Reload => reload_configuration()?,
    }

    Ok(())
}

fn show_status(storage: &Storage) -> Result<()> {
    println!("{}", "Phylax Status".bold().cyan());
    println!();

    let stats = storage.get_statistics();
    println!("{}: {}", "Database Path".bold(), "/var/lib/phylax/phylax.db");
    println!("{}: {}", "Total Events".bold(), stats.total_events);
    println!("{}: {}", "Total Alerts".bold(), stats.total_alerts);
    println!("{}: {}", "Active Rules".bold(), stats.active_rules);
    println!();

    Ok(())
}

fn show_logs(storage: &Storage, lines: usize) -> Result<()> {
    println!("{}", "Recent Logs".bold().cyan());
    println!();

    let logs = storage.get_recent_logs(lines)?;

    for log in logs {
        println!("[{}] {}", log.timestamp.format("%Y-%m-%d %H:%M:%S"), log.message);
    }

    Ok(())
}

fn show_events(storage: &Storage, event_type: Option<String>, limit: usize) -> Result<()> {
    println!("{}", "Security Events".bold().cyan());
    println!();

    let events = if let Some(et) = event_type {
        storage.get_events_by_type(&et, Some(limit))?
    } else {
        storage.get_recent_events(Some(limit))?
    };

    if events.is_empty() {
        println!("No events found");
        return Ok(());
    }

    let table_data: Vec<EventRow> = events.iter().map(|e| EventRow {
        id: e.id.to_string()[..8].to_string(),
        timestamp: e.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
        event_type: e.event_type.as_str().to_string(),
        severity: e.severity.as_str().to_string(),
        source: e.source.hostname.clone(),
    }).collect();

    let table = Table::new(table_data).to_string();
    println!("{}", table);

    Ok(())
}

fn show_incidents(storage: &Storage, limit: usize) -> Result<()> {
    println!("{}", "Security Incidents".bold().cyan());
    println!();

    let incidents = storage.get_incidents(Some(limit))?;

    if incidents.is_empty() {
        println!("No incidents found");
        return Ok(());
    }

    let table_data: Vec<IncidentRow> = incidents.iter().map(|i| IncidentRow {
        id: i.id.to_string()[..8].to_string(),
        timestamp: i.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
        title: i.title.clone(),
        severity: i.severity.as_str().to_string(),
        status: i.status.as_str().to_string(),
    }).collect();

    let table = Table::new(table_data).to_string();
    println!("{}", table);

    Ok(())
}

fn show_rules(storage: &Storage) -> Result<()> {
    println!("{}", "Active Rules".bold().cyan());
    println!();

    let rules = storage.get_rules()?;

    if rules.is_empty() {
        println!("No rules loaded");
        return Ok(());
    }

    for rule in rules {
        let status = if rule.enabled { "enabled".green() } else { "disabled".red() };
        println!("{}: {} ({})", rule.name.bold(), rule.event, status);
        println!("  Severity: {}", rule.severity);
        println!("  Conditions: {}", rule.conditions.len());
        println!("  Actions: {}", rule.actions.len());
        println!();
    }

    Ok(())
}

fn reload_configuration() -> Result<()> {
    println!("{}", "Reloading Configuration".bold().cyan());
    println!();

    #[cfg(unix)]
    {
        use std::process::Command;
        let output = Command::new("pkill")
            .args(["-HUP", "phylaxd"])
            .output();

        match output {
            Ok(_) => println!("{}", "Configuration reloaded successfully".green()),
            Err(e) => println!("{}: {}", "Failed to reload".red(), e),
        }
    }

    #[cfg(windows)]
    {
        println!("{}", "Reload not supported on Windows".yellow());
    }

    Ok(())
}

#[derive(Tabled)]
struct EventRow {
    id: String,
    timestamp: String,
    event_type: String,
    severity: String,
    source: String,
}

#[derive(Tabled)]
struct IncidentRow {
    id: String,
    timestamp: String,
    title: String,
    severity: String,
    status: String,
}