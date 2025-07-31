use anyhow::Result;
use clap::Parser;
use log::{info, warn, error};
use std::path::PathBuf;
use rdkafka::{
    config::{ClientConfig, FromClientConfig},
    consumer::{Consumer, StreamConsumer},
    Message,
};

use alerts::KafkaConfig;

#[derive(Parser)]
#[command(name = "kafka-consumer")]
#[command(about = "Consume and display Kafka messages with formatted output")]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    /// Consumer group ID (overrides config file)
    #[arg(short, long)]
    group_id: Option<String>,

    /// Kafka topic to consume from (overrides config file)
    #[arg(short, long)]
    topic: Option<String>,

    /// Auto offset reset: earliest, latest, none
    #[arg(short = 'o', long, default_value = "earliest")]
    offset_reset: String,

    /// Pretty print JSON messages
    #[arg(short, long)]
    pretty: bool,

    /// Show message metadata (offset, partition, timestamp)
    #[arg(short = 'd', long)]
    metadata: bool,

    /// Maximum number of messages to consume (0 = unlimited)
    #[arg(short, long, default_value = "0")]
    max_messages: u64,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
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

    info!("🚀 Starting Kafka Consumer");
    info!("Configuration: {:?}", args.config);

    // Load configuration
    let mut config = match KafkaConfig::from_file(&args.config.to_str().unwrap()) {
        Ok(config) => {
            info!("✅ Loaded configuration from {:?}", args.config);
            config
        }
        Err(e) => {
            warn!("Failed to load config from {:?}: {}. Using default configuration.", args.config, e);
            KafkaConfig::default()
        }
    };

    // Override config with command line arguments
    if let Some(group_id) = args.group_id {
        config.group_id = group_id;
    }
    if let Some(topic) = args.topic {
        config.topic = topic;
    }

    info!("📡 Kafka broker: {}", config.bootstrap_servers);
    info!("📂 Topic: {}", config.topic);
    info!("👥 Consumer group: {}", config.group_id);
    info!("⏮️  Offset reset: {}", args.offset_reset);

    // Create consumer configuration
    let mut consumer_config = ClientConfig::new();
    consumer_config
        .set("bootstrap.servers", &config.bootstrap_servers)
        .set("group.id", &config.group_id)
        .set("auto.offset.reset", &args.offset_reset)
        .set("enable.auto.commit", "true")
        .set("auto.commit.interval.ms", "1000");

    // Create consumer
    let consumer: StreamConsumer = StreamConsumer::from_config(&consumer_config)?;
    consumer.subscribe(&[&config.topic])?;

    info!("✅ Kafka consumer created and subscribed to topic: {}", config.topic);
    info!("📥 Starting to consume messages...");

    let mut count = 0;

    loop {
        // Check max messages limit
        if args.max_messages > 0 && count >= args.max_messages {
            info!("Reached maximum message limit: {}", args.max_messages);
            break;
        }

        match consumer.recv().await {
            Ok(msg) => {
                count += 1;

                // Show metadata if requested
                if args.metadata {
                    println!("━━━ Message #{} ━━━", count);
                    println!("📍 Partition: {}", msg.partition());
                    println!("📌 Offset: {}", msg.offset());
                    if let Some(timestamp) = msg.timestamp().to_millis() {
                        println!("⏰ Timestamp: {} ({})", timestamp, 
                                chrono::DateTime::from_timestamp_millis(timestamp)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                                .unwrap_or_else(|| "Invalid timestamp".to_string()));
                    }
                    if let Some(key) = msg.key_view::<str>() {
                        if let Ok(key_str) = key {
                            println!("🔑 Key: {}", key_str);
                        }
                    }
                    println!("📄 Payload:");
                }

                // Process message payload
                match msg.payload_view::<str>() {
                    Some(Ok(payload)) => {
                        if args.pretty {
                            // Try to parse as JSON and pretty print
                            match serde_json::from_str::<serde_json::Value>(payload) {
                                Ok(json_value) => {
                                    println!("{}", serde_json::to_string_pretty(&json_value)?);
                                }
                                Err(_) => {
                                    // Not valid JSON, print as-is
                                    println!("{}", payload);
                                }
                            }
                        } else {
                            println!("{}", payload);
                        }
                    }
                    Some(Err(e)) => {
                        error!("❌ Error deserializing message payload: {}", e);
                        continue;
                    }
                    None => {
                        warn!("⚠️  Empty message received");
                        if args.metadata {
                            println!("(empty payload)");
                        }
                    }
                }

                if args.metadata {
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!();
                }

                // Progress logging
                if args.verbose {
                    info!("📨 Processed message #{}", count);
                } else if count % 100 == 0 {
                    info!("📊 Processed {} messages", count);
                }
            }
            Err(e) => {
                error!("❌ Error receiving message: {}", e);
                if e.to_string().contains("timed out") {
                    // Timeout is normal, continue consuming
                    continue;
                } else {
                    // Other errors might be more serious
                    error!("💥 Consumer error, stopping: {}", e);
                    break;
                }
            }
        }
    }

    info!("🎉 Consumer finished");
    info!("📊 Total messages processed: {}", count);

    Ok(())
}