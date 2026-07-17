use phylax_common::{Error, Result, get_config_path, get_data_path, ensure_dir_exists, Config, Platform};
use phylax_storage::Storage;
use phylax_rules::RuleEngine;
use phylax_engine::{Engine, EngineConfig};
use phylax_policy::{PolicyEngine, PolicyConfig};
use phylax_logging::{Logger, LoggingConfig};
use phylax_alerts::{AlertManager, AlertContext, AlertConfig};
use phylax_collectors::{CollectorManager, create_collectors};
use phylax_responders::{ResponderManager, ResponderConfig};
use phylax_platform::PlatformDetector;
use crossbeam_channel::Sender;
use phylax_events::{Event, EventSource};
use clap::Parser;
use std::sync::Arc;
use std::path::PathBuf;
use signal_hook::consts::SIGTERM;
use signal_hook_tokio::Signals;
use tokio::select;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "/etc/phylax/phylax.toml")]
    config: String,

    #[arg(short, long)]
    foreground: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if !args.foreground {
        daemonize()?;
    }

    let config_path = PathBuf::from(args.config);
    let config = load_config(&config_path)?;

    let data_path = get_data_path()?;
    ensure_dir_exists(&data_path)?;

    let db_path = data_path.join("phylax.db");
    let storage = Arc::new(Storage::new(&db_path)?);

    let logging_config = LoggingConfig {
        level: config.logging.level.clone(),
        format: config.logging.format.clone(),
        log_path: Some(data_path.join("logs")),
        console_output: args.foreground,
        max_size_mb: config.logging.max_size_mb,
        max_files: config.logging.max_files,
    };
    let logger = Logger::new(logging_config)?;
    logger.log_info("Phylax daemon starting");

    let (event_sender, event_receiver) = crossbeam_channel::unbounded::<Event>();

    let mut rule_engine = RuleEngine::new();
    let rules_path = PathBuf::from("/etc/phylax/rules");
    if rules_path.exists() {
        let count = rule_engine.load_rules_from_dir(&rules_path)?;
        logger.log_info(&format!("Loaded {} rules", count));
    }

    let engine_config = EngineConfig {
        max_queue_size: config.daemon.max_events_per_second.unwrap_or(1000) as usize,
        worker_threads: config.daemon.worker_threads.unwrap_or(4),
    };
    let mut engine = Engine::new(storage.clone(), engine_config);
    engine.set_rule_engine(rule_engine);

    let policy_config = PolicyConfig::default();
    let policy_engine = PolicyEngine::new(policy_config);

    let responder_config = ResponderConfig {
        require_approval: config.responders.require_approval,
        allow_ip_blocking: config.responders.ip_blocking.is_some(),
        allow_process_termination: config.responders.process_termination
            .as_ref()
            .map(|p| p.allow_termination)
            .unwrap_or(false),
        allow_file_quarantine: config.responders.file_quarantine.is_some(),
        allow_account_disable: false,
    };
    let responder_manager = ResponderManager::new(responder_config);

    let alert_config = AlertConfig {
        enabled_providers: config.alerts.enabled.clone(),
        email: config.alerts.email.clone(),
        webhook: config.alerts.webhook.clone(),
    };
    let mut alert_manager = AlertManager::new();

    let collectors = create_collectors(event_sender.clone());
    let mut collector_manager = CollectorManager::new(collectors);

    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let running_clone = running.clone();

    let signals = Signals::new([SIGTERM])?;

    tokio::spawn(async move {
        #[allow(clippy::never_loop)]
        for sig in signals.forever() {
            eprintln!("Received signal {:?}, shutting down", sig);
            running_clone.store(false, std::sync::atomic::Ordering::SeqCst);
        }
    });

    collector_manager.start()?;
    logger.log_info("Collectors started");

    let decision_receiver = engine.get_decision_receiver();

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        select! {
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                if let Ok(event) = event_receiver.try_recv() {
                    logger.log_event(&event);
                    
                    if let Err(e) = engine.process_event(event) {
                        logger.log_error("Failed to process event", &e);
                    }
                }

                if let Ok(decision) = decision_receiver.try_recv() {
                    let policy_result = policy_engine.evaluate_decision(&decision);

                    if policy_engine.should_alert(&decision) {
                        let alert_context = AlertContext {
                            decision: decision.clone(),
                            event_id: decision.event_id,
                            severity: decision.threat_level.into(),
                            title: format!("Security Alert: {}", decision.rule_name),
                            description: Some(format!("Threat level: {}", decision.threat_level.as_str())),
                        };

                        if let Err(e) = alert_manager.send_alert(&alert_context).await {
                            logger.log_error("Failed to send alert", &e);
                        }
                    }

                    if policy_engine.should_execute_action(&decision) {
                        match responder_manager.execute_decision(&decision) {
                            Ok(results) => {
                                for (responder, result) in results {
                                    logger.log_info(&format!(
                                        "Responder {}: {}",
                                        responder,
                                        if result.success { "success" } else { "failed" }
                                    ));
                                }
                            }
                            Err(e) => logger.log_error("Failed to execute responders", &e),
                        }
                    }
                }
            }
        }
    }

    collector_manager.stop()?;
    logger.log_info("Phylax daemon stopped");

    Ok(())
}

fn load_config(path: &PathBuf) -> Result<Config> {
    if path.exists() {
        Config::load_config(path)
    } else {
        Ok(Config::default())
    }
}

fn daemonize() -> Result<()> {
    #[cfg(unix)]
    {
        unsafe {
            if libc::fork() > 0 {
                std::process::exit(0);
            }
        }

        unsafe {
            libc::setsid();
        }

        unsafe {
            if libc::fork() > 0 {
                std::process::exit(0);
            }
        }

        std::fs::File::create("/dev/null")?;
    }

    #[cfg(windows)]
    {
        eprintln!("Daemon mode not supported on Windows, running in foreground");
    }

    Ok(())
}