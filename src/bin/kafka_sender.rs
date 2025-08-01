use anyhow::Result;
use clap::Parser;
use log::{info, warn, error};
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::fs::File;

use alerts::{AlertMessage, EdrAlert, NgavAlert, KafkaConfig, KafkaProducer};
use alerts::kafka::{KafkaProducerConfig, KafkaConsumerConfig};

#[derive(Parser)]
#[command(name = "kafka-sender")]
#[command(about = "Send JSONL data to Kafka with configurable rate and data type")]
struct Args {
    /// Kafka bootstrap servers (e.g., localhost:9092)
    #[arg(short = 'b', long)]
    bootstrap_servers: String,

    /// Kafka topic to send messages to
    #[arg(short = 'o', long)]
    topic: String,

    /// Kafka consumer group ID
    #[arg(short = 'g', long)]
    group_id: String,

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
    info!("📡 Kafka broker: {}", args.bootstrap_servers);
    info!("📂 Topic: {}", args.topic);
    info!("👥 Group ID: {}", args.group_id);
    info!("📄 Data file: {:?}", args.data);
    info!("📊 Data type: {:?}", args.data_type);
    info!("⚡ Send rate: {} messages/second", args.rate);

    // Create configuration from command line args with default producer/consumer settings
    let config = KafkaConfig {
        bootstrap_servers: args.bootstrap_servers.clone(),
        topic: args.topic.clone(),
        group_id: args.group_id.clone(),
        producer: KafkaProducerConfig {
            message_timeout_ms: 5000,
            request_timeout_ms: 5000,
            retry_backoff_ms: 100,
            retries: 3,
        },
        consumer: KafkaConsumerConfig {
            auto_offset_reset: "earliest".to_string(),
            enable_auto_commit: true,
            auto_commit_interval_ms: 1000,
        },
    };

    // Create producer
    let producer = KafkaProducer::new(config).await?;
    info!("✅ Kafka producer created successfully");

    // Open data file
    let file = File::open(&args.data).await?;
    info!("📂 Opened file: {:?}", args.data);
    
    // Count total lines first
    let file_for_count = File::open(&args.data).await?;
    let reader_for_count = BufReader::new(file_for_count);
    let mut lines_for_count = reader_for_count.lines();
    let mut total_lines = 0;
    while let Some(line) = lines_for_count.next_line().await? {
        if !line.trim().is_empty() {
            total_lines += 1;
        }
    }
    info!("📄 Total non-empty lines in file: {}", total_lines);
    
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
    let mut line_number = 0;

    while let Some(line) = lines.next_line().await? {
        line_number += 1;
        if line.trim().is_empty() {
            if args.verbose {
                info!("⏭️  Skipping empty line #{}", line_number);
            }
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
                    info!("✅ Sent message #{} (line #{})", count, line_number);
                } else if count % 100 == 0 {
                    info!("📊 Sent {} messages ({} success, {} errors)", count, success_count, error_count);
                }
            }
            Err(e) => {
                error_count += 1;
                error!("❌ Failed to send message #{} (line #{}): {}", count, line_number, e);
                if args.verbose {
                    error!("📝 Failed line content: {}", line.chars().take(200).collect::<String>());
                }
            }
        }

        // Rate limiting
        if delay > Duration::from_millis(0) {
            tokio::time::sleep(delay).await;
        }
    }

    info!("🎉 Completed sending messages");
    info!("📊 Final stats: {} total, {} success, {} errors", count, success_count, error_count);
    
    // Flush any pending messages
    info!("⏳ Flushing producer to ensure all messages are sent...");
    producer.flush(Duration::from_secs(10))?;
    info!("✅ All messages have been flushed to Kafka");

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