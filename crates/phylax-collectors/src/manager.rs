use phylax_common::{Error, Result};
use crossbeam_channel::Sender;
use phylax_events::Event;
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::Collector;

pub struct CollectorManager {
    collectors: Vec<Box<dyn Collector + Send>>,
    running: Arc<AtomicBool>,
}

impl CollectorManager {
    pub fn new(collectors: Vec<Box<dyn Collector + Send>>) -> Self {
        CollectorManager {
            collectors,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self) -> Result<()> {
        self.running.store(true, Ordering::SeqCst);

        for collector in &mut self.collectors {
            let running = self.running.clone();
            let name = collector.name();

            thread::spawn(move || {
                while running.load(Ordering::SeqCst) {
                    if let Err(e) = collector.start() {
                        eprintln!("Collector {} error: {}", name, e);
                        break;
                    }
                }
            });
        }

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);

        for collector in &mut self.collectors {
            if let Err(e) = collector.stop() {
                eprintln!("Failed to stop collector: {}", e);
            }
        }

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn collector_count(&self) -> usize {
        self.collectors.len()
    }
}
