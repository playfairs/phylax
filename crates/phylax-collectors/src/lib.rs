use phylax_common::{Error, Result, Severity};
use phylax_events::{Event, EventSource, EventType};
use phylax_platform::Platform;
use crossbeam_channel::Sender;
use std::time::Duration;

pub mod ssh;
pub mod process;
pub mod filesystem;
pub mod manager;

pub use ssh::*;
pub use process::*;
pub use filesystem::*;
pub use manager::*;

pub trait Collector {
    fn name(&self) -> &'static str;
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn is_running(&self) -> bool;
}

pub struct CollectorConfig {
    pub enabled: bool,
    pub interval: Duration,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        CollectorConfig {
            enabled: true,
            interval: Duration::from_secs(5),
        }
    }
}

pub fn create_collectors(event_sender: Sender<Event>) -> Vec<Box<dyn Collector + Send>> {
    let mut collectors: Vec<Box<dyn Collector + Send>> = Vec::new();

    match Platform::current() {
        Platform::Linux => {
            collectors.push(Box::new(SshCollector::new(event_sender.clone())));
            collectors.push(Box::new(ProcessCollector::new(event_sender.clone())));
            collectors.push(Box::new(FilesystemCollector::new(event_sender.clone())));
        }
        Platform::MacOS => {
            collectors.push(Box::new(SshCollector::new(event_sender.clone())));
            collectors.push(Box::new(ProcessCollector::new(event_sender.clone())));
            collectors.push(Box::new(FilesystemCollector::new(event_sender.clone())));
        }
        Platform::Windows => {
            collectors.push(Box::new(ProcessCollector::new(event_sender.clone())));
        }
    }

    collectors
}