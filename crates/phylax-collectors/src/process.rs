use phylax_common::{Error, Result, Severity};
use phylax_events::{Event, EventSource, EventType};
use phylax_platform::{Platform, ProcessInfo};
use crossbeam_channel::Sender;
use std::collections::HashMap;
use std::time::Duration;
use std::thread;

pub struct ProcessCollector {
    event_sender: Sender<Event>,
    running: bool,
    known_processes: HashMap<u32, String>,
}

impl ProcessCollector {
    pub fn new(event_sender: Sender<Event>) -> Self {
        ProcessCollector {
            event_sender,
            running: false,
            known_processes: HashMap::new(),
        }
    }

    fn scan_processes(&mut self) -> Result<()> {
        let processes = self.get_process_list()?;

        for process in processes {
            if !self.known_processes.contains_key(&process.pid) {
                self.send_process_created_event(&process)?;
                self.known_processes.insert(process.pid, process.name.clone());
            }
        }

        self.check_terminated_processes(&processes)?;

        Ok(())
    }

    fn get_process_list(&self) -> Result<Vec<ProcessInfo>> {
        match Platform::current() {
            Platform::Linux => self.get_linux_processes(),
            Platform::MacOS => self.get_macos_processes(),
            Platform::Windows => self.get_windows_processes(),
        }
    }

    #[cfg(target_os = "linux")]
    fn get_linux_processes(&self) -> Result<Vec<ProcessInfo>> {
        let mut processes = Vec::new();

        for proc_entry in std::fs::read_dir("/proc")
            .map_err(|e| Error::CollectorError(format!("Failed to read /proc: {}", e)))?
        {
            let entry = proc_entry
                .map_err(|e| Error::CollectorError(format!("Failed to read proc entry: {}", e)))?;

            if let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() {
                if let Some(info) = self.read_linux_process_info(pid) {
                    processes.push(info);
                }
            }
        }

        Ok(processes)
    }

    #[cfg(target_os = "linux")]
    fn read_linux_process_info(&self, pid: u32) -> Option<ProcessInfo> {
        let cmdline_path = format!("/proc/{}/cmdline", pid);
        let stat_path = format!("/proc/{}/stat", pid);

        let cmdline = std::fs::read_to_string(&cmdline_path).ok()?
            .replace('\0', " ");

        let stat = std::fs::read_to_string(&stat_path).ok()?;
        let parts: Vec<&str> = stat.split_whitespace().collect();
        let comm = parts.get(1)?.to_string();
        let ppid = parts.get(3).and_then(|s| s.parse().ok());

        Some(ProcessInfo {
            pid,
            ppid,
            name: comm,
            command: cmdline,
            user: None,
        })
    }

    #[cfg(target_os = "macos")]
    fn get_macos_processes(&self) -> Result<Vec<ProcessInfo>> {
        let output = std::process::Command::new("ps")
            .args(["-axo", "pid,ppid,comm,args"])
            .output()
            .map_err(|e| Error::CollectorError(format!("Failed to list processes: {}", e)))?;

        let mut processes = Vec::new();
        let lines = String::from_utf8_lossy(&output.stdout);

        for line in lines.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let pid = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
                let ppid = parts.get(1).and_then(|s| s.parse().ok());
                let name = parts.get(2).unwrap_or(&"unknown").to_string();
                let command = parts[3..].join(" ");

                processes.push(ProcessInfo {
                    pid,
                    ppid,
                    name,
                    command,
                    user: None,
                });
            }
        }

        Ok(processes)
    }

    #[cfg(target_os = "windows")]
    fn get_windows_processes(&self) -> Result<Vec<ProcessInfo>> {
        let output = std::process::Command::new("tasklist")
            .args(["/FO", "CSV", "/NH"])
            .output()
            .map_err(|e| Error::CollectorError(format!("Failed to list processes: {}", e)))?;

        let mut processes = Vec::new();
        let lines = String::from_utf8_lossy(&output.stdout);

        for line in lines.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                let name = parts.get(0).map(|s| s.trim_matches('"')).unwrap_or("unknown");
                let pid = parts.get(1).and_then(|s| s.trim_matches('"').parse().ok()).unwrap_or(0);

                processes.push(ProcessInfo {
                    pid,
                    ppid: None,
                    name: name.to_string(),
                    command: name.to_string(),
                    user: None,
                });
            }
        }

        Ok(processes)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    fn get_linux_processes(&self) -> Result<Vec<ProcessInfo>> {
        Ok(Vec::new())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    fn get_macos_processes(&self) -> Result<Vec<ProcessInfo>> {
        Ok(Vec::new())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    fn get_windows_processes(&self) -> Result<Vec<ProcessInfo>> {
        Ok(Vec::new())
    }

    fn send_process_created_event(&self, process: &ProcessInfo) -> Result<()> {
        let source = EventSource::new(hostname())
            .with_process_id(process.pid)
            .with_process_name(process.name.clone());

        let event = Event::new(EventType::ProcessCreated, Severity::Low, source)
            .with_metadata("command".to_string(), serde_json::json!(process.command))
            .with_metadata("ppid".to_string(), serde_json::json!(process.ppid));

        self.event_sender.send(event)
            .map_err(|e| Error::CollectorError(format!("Failed to send event: {}", e)))?;

        Ok(())
    }

    fn check_terminated_processes(&mut self, current_processes: &[ProcessInfo]) -> Result<()> {
        let current_pids: std::collections::HashSet<u32> = current_processes.iter().map(|p| p.pid).collect();

        let mut terminated = Vec::new();
        for (&pid, name) in &self.known_processes {
            if !current_pids.contains(&pid) {
                terminated.push((pid, name.clone()));
            }
        }

        for (pid, name) in terminated {
            self.send_process_terminated_event(pid, &name)?;
            self.known_processes.remove(&pid);
        }

        Ok(())
    }

    fn send_process_terminated_event(&self, pid: u32, name: &str) -> Result<()> {
        let source = EventSource::new(hostname())
            .with_process_id(pid)
            .with_process_name(name.to_string());

        let event = Event::new(EventType::ProcessTerminated, Severity::Low, source);

        self.event_sender.send(event)
            .map_err(|e| Error::CollectorError(format!("Failed to send event: {}", e)))?;

        Ok(())
    }
}

impl Collector for ProcessCollector {
    fn name(&self) -> &'static str {
        "process"
    }

    fn start(&mut self) -> Result<()> {
        self.running = true;

        while self.running {
            if let Err(e) = self.scan_processes() {
                eprintln!("Failed to scan processes: {}", e);
            }
            thread::sleep(Duration::from_secs(5));
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
