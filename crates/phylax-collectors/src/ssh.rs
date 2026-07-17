use phylax_common::{Error, Result, Severity};
use phylax_events::{Event, EventSource, EventType};
use phylax_platform::Platform;
use crossbeam_channel::Sender;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct SshCollector {
    event_sender: Sender<Event>,
    running: bool,
    log_paths: Vec<PathBuf>,
}

impl SshCollector {
    pub fn new(event_sender: Sender<Event>) -> Self {
        let log_paths = match Platform::current() {
            Platform::Linux => vec![
                PathBuf::from("/var/log/auth.log"),
                PathBuf::from("/var/log/secure"),
            ],
            Platform::MacOS => vec![
                PathBuf::from("/var/log/system.log"),
                PathBuf::from("/var/log/auth.log"),
            ],
            Platform::Windows => vec![],
        };

        SshCollector {
            event_sender,
            running: false,
            log_paths,
        }
    }

    fn monitor_ssh_log(&self, path: &PathBuf) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let file = File::open(path)
            .map_err(|e| Error::CollectorError(format!("Failed to open SSH log: {}", e)))?;
        
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line.map_err(|e| Error::CollectorError(format!("Failed to read line: {}", e)))?;
            
            if line.contains("Accepted") || line.contains("session opened") {
                self.send_ssh_login_event(&line, path)?;
            } else if line.contains("Failed") || line.contains("authentication failure") {
                self.send_ssh_failure_event(&line, path)?;
            }
        }

        Ok(())
    }

    fn send_ssh_login_event(&self, line: &str, path: &PathBuf) -> Result<()> {
        let source = EventSource::new(hostname())
            .with_user(extract_user(line))
            .with_ip_address(extract_ip(line));

        let event = Event::new(EventType::SshLogin, Severity::Medium, source)
            .with_metadata("log_file".to_string(), serde_json::json!(path.to_string_lossy()))
            .with_metadata("log_line".to_string(), serde_json::json!(line));

        self.event_sender.send(event)
            .map_err(|e| Error::CollectorError(format!("Failed to send event: {}", e)))?;

        Ok(())
    }

    fn send_ssh_failure_event(&self, line: &str, path: &PathBuf) -> Result<()> {
        let source = EventSource::new(hostname())
            .with_user(extract_user(line))
            .with_ip_address(extract_ip(line));

        let event = Event::new(EventType::AuthenticationFailure, Severity::High, source)
            .with_metadata("log_file".to_string(), serde_json::json!(path.to_string_lossy()))
            .with_metadata("log_line".to_string(), serde_json::json!(line));

        self.event_sender.send(event)
            .map_err(|e| Error::CollectorError(format!("Failed to send event: {}", e)))?;

        Ok(())
    }
}

impl Collector for SshCollector {
    fn name(&self) -> &'static str {
        "ssh"
    }

    fn start(&mut self) -> Result<()> {
        self.running = true;

        for log_path in &self.log_paths {
            if let Err(e) = self.monitor_ssh_log(log_path) {
                eprintln!("Failed to monitor SSH log {}: {}", log_path.display(), e);
            }
        }

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

fn extract_user(line: &str) -> Option<String> {
    if line.contains("for ") {
        let parts: Vec<&str> = line.split("for ").collect();
        if parts.len() > 1 {
            let user_part = parts[1].split_whitespace().next()?;
            return Some(user_part.to_string());
        }
    }
    None
}

fn extract_ip(line: &str) -> Option<String> {
    let ip_regex = regex::Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").ok()?;
    ip_regex.find(line).map(|m| m.as_str().to_string())
}
