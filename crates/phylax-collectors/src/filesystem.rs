use phylax_common::{Error, Result, Severity};
use phylax_events::{Event, EventSource, EventType};
use phylax_platform::Platform;
use crossbeam_channel::Sender;
use std::path::PathBuf;
use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use std::sync::mpsc::channel;
use std::time::Duration;

pub struct FilesystemCollector {
    event_sender: Sender<Event>,
    running: bool,
    watch_paths: Vec<PathBuf>,
}

impl FilesystemCollector {
    pub fn new(event_sender: Sender<Event>) -> Self {
        let watch_paths = match Platform::current() {
            Platform::Linux => vec![
                PathBuf::from("/etc"),
                PathBuf::from("/home"),
            ],
            Platform::MacOS => vec![
                PathBuf::from("/etc"),
                PathBuf::from("/Users"),
            ],
            Platform::Windows => vec![
                PathBuf::from("C:\\\\Windows\\\\System32"),
            ],
        };

        FilesystemCollector {
            event_sender,
            running: false,
            watch_paths,
        }
    }

    fn start_watcher(&self) -> Result<()> {
        let (tx, rx) = channel();

        let mut watcher: Watcher = watcher(tx, Duration::from_secs(2))
            .map_err(|e| Error::CollectorError(format!("Failed to create watcher: {}", e)))?;

        for path in &self.watch_paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)
                    .map_err(|e| Error::CollectorError(format!("Failed to watch {}: {}", path.display(), e)))?;
            }
        }

        while self.running {
            match rx.recv() {
                Ok(event) => {
                    if let Err(e) = self.handle_filesystem_event(&event) {
                        eprintln!("Failed to handle filesystem event: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Watcher error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    fn handle_filesystem_event(&self, event: &DebouncedEvent) -> Result<()> {
        match event {
            DebouncedEvent::Create(path) => {
                self.send_filesystem_create_event(path)?;
            }
            DebouncedEvent::Write(path) => {
                self.send_filesystem_modify_event(path)?;
            }
            DebouncedEvent::Remove(path) => {
                self.send_filesystem_delete_event(path)?;
            }
            DebouncedEvent::Rename(src, dest) => {
                self.send_filesystem_modify_event(dest)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn send_filesystem_create_event(&self, path: &PathBuf) -> Result<()> {
        let source = EventSource::new(hostname());

        let event = Event::new(EventType::FilesystemCreate, Severity::Low, source)
            .with_metadata("path".to_string(), serde_json::json!(path.to_string_lossy()))
            .with_metadata("file_type".to_string(), serde_json::json!(get_file_type(path)));

        self.event_sender.send(event)
            .map_err(|e| Error::CollectorError(format!("Failed to send event: {}", e)))?;

        Ok(())
    }

    fn send_filesystem_modify_event(&self, path: &PathBuf) -> Result<()> {
        let source = EventSource::new(hostname());

        let event = Event::new(EventType::FilesystemModify, Severity::Low, source)
            .with_metadata("path".to_string(), serde_json::json!(path.to_string_lossy()))
            .with_metadata("file_type".to_string(), serde_json::json!(get_file_type(path)));

        self.event_sender.send(event)
            .map_err(|e| Error::CollectorError(format!("Failed to send event: {}", e)))?;

        Ok(())
    }

    fn send_filesystem_delete_event(&self, path: &PathBuf) -> Result<()> {
        let source = EventSource::new(hostname());

        let event = Event::new(EventType::FilesystemDelete, Severity::Medium, source)
            .with_metadata("path".to_string(), serde_json::json!(path.to_string_lossy()));

        self.event_sender.send(event)
            .map_err(|e| Error::CollectorError(format!("Failed to send event: {}", e)))?;

        Ok(())
    }
}

impl Collector for FilesystemCollector {
    fn name(&self) -> &'static str {
        "filesystem"
    }

    fn start(&mut self) -> Result<()> {
        self.running = true;
        self.start_watcher()
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

fn get_file_type(path: &PathBuf) -> String {
    if path.is_dir() {
        "directory".to_string()
    } else if path.is_file() {
        "file".to_string()
    } else {
        "unknown".to_string()
    }
}
