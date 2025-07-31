use anyhow::Result;
use clap::Parser;
use log::{info, warn, error};
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::fs::File;

use alerts::{AlertMessage, EdrAlert, NgavAlert, KafkaConfig, KafkaProducer};

#[derive(Parser)]
#[command(name = "kafka-sender")]
#[command(about = "Send JSONL data to Kafka with configurable rate and data type")]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    /// Path to JSONL data file
    #[arg(short, long)]
    data: PathBuf,

    /// Data type: alert, edr, ngav
    #[arg(short = 't', long, value_enum)]
    data_type: DataType,

    /// Send rate in messages per second
    #[arg(short, long, default_value = "10")]
    rate: u64,

    /// Maximum number of messages to send (0 = unlimited)
    #[arg(short, long, default_value = "0")]
    max_messages: u64,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum DataType {
    Alert,
    Edr,
    Ngav,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    if args.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    info!("🚀 Starting Kafka Sender");
    info!("Configuration: {:?}", args.config);
    info!("Data file: {:?}", args.data);
    info!("Data type: {:?}", args.data_type);
    info!("Send rate: {} messages/second", args.rate);

    // Load configuration
    let config = match KafkaConfig::from_file(&args.config.to_str().unwrap()) {
        Ok(config) => {
            info!("✅ Loaded configuration from {:?}", args.config);
            info!("📡 Kafka broker: {}", config.bootstrap_servers);
            info!("📂 Topic: {}", config.topic);
            config
        }
        Err(e) => {
            warn!("Failed to load config from {:?}: {}. Using default configuration.", args.config, e);
            let default_config = KafkaConfig::default();
            info!("📡 Using default Kafka broker: {}", default_config.bootstrap_servers);
            default_config
        }
    };

    // Create producer
    let producer = KafkaProducer::new(config).await?;
    info!("✅ Kafka producer created successfully");

    // Open data file
    let file = File::open(&args.data).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Calculate delay between messages
    let delay = if args.rate > 0 {
        Duration::from_millis(1000 / args.rate)
    } else {
        Duration::from_millis(0)
    };

    info!("📤 Starting to send messages with delay: {:?}", delay);

    let mut count = 0;
    let mut success_count = 0;
    let mut error_count = 0;

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        // Check max messages limit
        if args.max_messages > 0 && count >= args.max_messages {
            info!("Reached maximum message limit: {}", args.max_messages);
            break;
        }

        count += 1;

        // Parse and send message based on data type
        let result = match args.data_type {
            DataType::Alert => send_alert_message(&producer, &line).await,
            DataType::Edr => send_edr_message(&producer, &line).await,
            DataType::Ngav => send_ngav_message(&producer, &line).await,
        };

        match result {
            Ok(_) => {
                success_count += 1;
                if args.verbose {
                    info!("✅ Sent message #{}", count);
                } else if count % 100 == 0 {
                    info!("📊 Sent {} messages ({} success, {} errors)", count, success_count, error_count);
                }
            }
            Err(e) => {
                error_count += 1;
                error!("❌ Failed to send message #{}: {}", count, e);
            }
        }

        // Rate limiting
        if delay > Duration::from_millis(0) {
            tokio::time::sleep(delay).await;
        }
    }

    info!("🎉 Completed sending messages");
    info!("📊 Final stats: {} total, {} success, {} errors", count, success_count, error_count);

    Ok(())
}

async fn send_alert_message(producer: &KafkaProducer, line: &str) -> Result<()> {
    match serde_json::from_str::<AlertMessage>(line) {
        Ok(alert) => {
            producer.send_alert(alert).await
        }
        Err(e) => {
            // Try to create a generic alert from the line
            warn!("Failed to parse as AlertMessage: {}. Creating generic alert.", e);
            let alert = AlertMessage::info(
                format!("generic-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)),
                line.to_string(),
            );
            producer.send_alert(alert).await
        }
    }
}

async fn send_edr_message(producer: &KafkaProducer, line: &str) -> Result<()> {
    let edr_alert: EdrAlert = serde_json::from_str(line)?;
    producer.send_edr_alert(edr_alert).await
}

async fn send_ngav_message(producer: &KafkaProducer, line: &str) -> Result<()> {
    let ngav_alert: NgavAlert = serde_json::from_str(line)?;
    producer.send_ngav_alert(ngav_alert).await
}