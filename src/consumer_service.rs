use anyhow::Result;
use log::{error, info, warn, debug};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    database::{Database, KafkaConfigRow},
    clickhouse::{ClickHouseConnection, CommonAlert, EdrAlertRow, NgavAlertRow},
    edr_alert::EdrAlert,
    ngav_alert::NgavAlert,
};

pub struct ConsumerService {
    db: Arc<Database>,
    clickhouse: Arc<ClickHouseConnection>,
    consumers: Arc<RwLock<HashMap<Uuid, ConsumerInstance>>>,
    data_source_mapping: Arc<RwLock<HashMap<Uuid, String>>>, // kafka_config_id -> data_type
    message_sender: broadcast::Sender<KafkaMessage>,
}

#[derive(Debug, Clone)]
pub struct KafkaMessage {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub payload: String,
    pub timestamp: Option<i64>,
    pub kafka_config_id: Uuid,
    pub data_type: Option<String>,
}

struct ConsumerInstance {
    handle: JoinHandle<()>,
    config_id: Uuid,
    topic: String,
}

impl ConsumerService {
    pub async fn new(
        db: Arc<Database>,
        clickhouse: Arc<ClickHouseConnection>,
    ) -> Result<Self> {
        let (message_sender, _) = broadcast::channel(1000);
        
        let service = Self {
            db,
            clickhouse,
            consumers: Arc::new(RwLock::new(HashMap::new())),
            data_source_mapping: Arc::new(RwLock::new(HashMap::new())),
            message_sender,
        };

        // Initialize data source mapping
        service.refresh_data_source_mapping().await?;

        Ok(service)
    }

    pub fn get_message_receiver(&self) -> broadcast::Receiver<KafkaMessage> {
        self.message_sender.subscribe()
    }

    pub async fn refresh_data_source_mapping(&self) -> Result<()> {
        let configs = self.db.get_data_source_configs().await?;
        let mut mapping = self.data_source_mapping.write().await;
        
        mapping.clear();
        for config in configs {
            if let (Some(kafka_config_id), Some(data_type)) = (
                config.get("kafka_config_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()),
                config.get("data_type").and_then(|v| v.as_str())
            ) {
                mapping.insert(kafka_config_id, data_type.to_string());
            }
        }
        
        info!("Updated data source mapping with {} entries", mapping.len());
        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting consumer service...");
        
        // Get all active Kafka configurations
        let active_configs = self.db.get_active_kafka_configs().await?;
        
        if active_configs.is_empty() {
            warn!("No active Kafka configurations found");
            return Ok(());
        }

        // Start consumers for each active configuration
        for config in active_configs {
            self.start_consumer(config).await?;
        }

        info!("Consumer service started with {} consumers", self.consumers.read().await.len());
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping consumer service...");
        
        let mut consumers = self.consumers.write().await;
        for (config_id, instance) in consumers.drain() {
            info!("Stopping consumer for config {}", config_id);
            instance.handle.abort();
        }
        
        info!("Consumer service stopped");
        Ok(())
    }

    pub async fn refresh_consumers(&self) -> Result<()> {
        info!("Refreshing consumers...");
        
        // Stop all existing consumers
        self.stop().await?;
        
        // Refresh data source mapping
        self.refresh_data_source_mapping().await?;
        
        // Start consumers again
        self.start().await?;
        
        Ok(())
    }

    async fn start_consumer(&self, config: KafkaConfigRow) -> Result<()> {
        info!(
            "Starting consumer for config '{}' ({}:{})", 
            config.name, config.bootstrap_servers, config.topic
        );

        // Create Kafka consumer
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", &config.group_id)
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", config.enable_auto_commit.to_string())
            .set("auto.commit.interval.ms", config.auto_commit_interval_ms.to_string())
            .set("auto.offset.reset", &config.auto_offset_reset)
            .create()?;

        consumer.subscribe(&[&config.topic])?;

        // Clone necessary data for the async task
        let config_id = config.id;
        let topic = config.topic.clone();
        let clickhouse = Arc::clone(&self.clickhouse);
        let data_source_mapping = Arc::clone(&self.data_source_mapping);
        let message_sender = self.message_sender.clone();

        // Spawn consumer task
        let handle = tokio::spawn(async move {
            info!("Consumer task started for config {}", config_id);
            
            loop {
                match consumer.recv().await {
                    Ok(message) => {
                        let payload = match message.payload() {
                            Some(bytes) => {
                                match String::from_utf8_lossy(bytes).into_owned() {
                                    payload => {
                                        // Check if the conversion resulted in replacement characters
                                        if payload.contains('\u{FFFD}') {
                                            warn!("Message payload contains invalid UTF-8 characters, using lossy conversion");
                                        }
                                        payload
                                    }
                                }
                            }
                            None => {
                                warn!("Empty message payload");
                                continue;
                            }
                        };

                        let kafka_message = KafkaMessage {
                            topic: message.topic().to_string(),
                            partition: message.partition(),
                            offset: message.offset(),
                            payload: payload.clone(),
                            timestamp: message.timestamp().to_millis(),
                            kafka_config_id: config_id,
                            data_type: data_source_mapping.read().await.get(&config_id).cloned(),
                        };

                        // Send message for real-time monitoring
                        if let Err(e) = message_sender.send(kafka_message.clone()) {
                            debug!("No active message receivers: {}", e);
                        }

                        // Process and store the message
                        if let Err(e) = Self::process_message(
                            &kafka_message,
                            Arc::clone(&clickhouse),
                        ).await {
                            error!("Error processing message: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Kafka receive error for config {}: {}", config_id, e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });

        // Store the consumer instance
        let instance = ConsumerInstance {
            handle,
            config_id,
            topic,
        };

        self.consumers.write().await.insert(config.id, instance);
        
        info!("Consumer started for config '{}'", config.name);
        Ok(())
    }

    async fn process_message(
        kafka_message: &KafkaMessage,
        clickhouse: Arc<ClickHouseConnection>,
    ) -> Result<()> {
        debug!("Processing message from topic: {}", kafka_message.topic);

        // Parse JSON payload
        let json_value: Value = serde_json::from_str(&kafka_message.payload)?;
        
        // Determine data type and process accordingly
        match kafka_message.data_type.as_deref() {
            Some("edr") => {
                if let Ok(edr_alert) = serde_json::from_value::<EdrAlert>(json_value) {
                    Self::store_edr_alert(&edr_alert, kafka_message, &clickhouse).await?;
                } else {
                    debug!("Message from EDR topic doesn't match EDR format, skipping");
                }
            }
            Some("ngav") => {
                if let Ok(ngav_alert) = serde_json::from_value::<NgavAlert>(json_value) {
                    Self::store_ngav_alert(&ngav_alert, kafka_message, &clickhouse).await?;
                } else {
                    debug!("Message from NGAV topic doesn't match NGAV format, skipping");
                }
            }
            _ => {
                // Try to detect alert type automatically based on structure
                if Self::looks_like_edr(&json_value) {
                    if let Ok(edr_alert) = serde_json::from_value::<EdrAlert>(json_value) {
                        Self::store_edr_alert(&edr_alert, kafka_message, &clickhouse).await?;
                    } else {
                        debug!("Message looks like EDR but failed to parse");
                    }
                } else if Self::looks_like_ngav(&json_value) {
                    if let Ok(ngav_alert) = serde_json::from_value::<NgavAlert>(json_value) {
                        Self::store_ngav_alert(&ngav_alert, kafka_message, &clickhouse).await?;
                    } else {
                        debug!("Message looks like NGAV but failed to parse");
                    }
                } else {
                    debug!("Unable to identify message type, skipping");
                }
            }
        }

        Ok(())
    }

    fn looks_like_edr(json_value: &Value) -> bool {
        // Check for EDR-specific fields
        json_value.get("report_id").is_some() &&
        json_value.get("device_id").is_some() &&
        json_value.get("process_path").is_some() &&
        json_value.get("parent_path").is_some()
    }

    fn looks_like_ngav(json_value: &Value) -> bool {
        // Check for NGAV-specific fields
        json_value.get("threat_id").is_some() &&
        json_value.get("policy_id").is_some() &&
        json_value.get("workflow").is_some() &&
        json_value.get("reason").is_some()
    }

    async fn store_edr_alert(
        edr_alert: &EdrAlert,
        kafka_message: &KafkaMessage,
        clickhouse: &ClickHouseConnection,
    ) -> Result<()> {
        debug!("Storing EDR alert: {}", edr_alert.get_alert_key());

        // Create common alert
        let mut common_alert = CommonAlert::from(edr_alert);
        common_alert.kafka_topic = kafka_message.topic.clone();
        common_alert.kafka_partition = kafka_message.partition as u32;
        common_alert.kafka_offset = kafka_message.offset as u64;

        // Create EDR specific alert
        let mut edr_row = EdrAlertRow::from(edr_alert);
        edr_row.kafka_topic = kafka_message.topic.clone();
        edr_row.kafka_partition = kafka_message.partition as u32;
        edr_row.kafka_offset = kafka_message.offset as u64;

        // Store both alerts
        clickhouse.insert_common_alert(&common_alert).await?;
        clickhouse.insert_edr_alert(&edr_row).await?;

        info!(
            "Stored EDR alert: {} (severity: {})",
            edr_alert.get_alert_key(),
            edr_alert.get_severity_level()
        );

        Ok(())
    }

    async fn store_ngav_alert(
        ngav_alert: &NgavAlert,
        kafka_message: &KafkaMessage,
        clickhouse: &ClickHouseConnection,
    ) -> Result<()> {
        debug!("Storing NGAV alert: {}", ngav_alert.get_alert_key());

        // Create common alert
        let mut common_alert = CommonAlert::from(ngav_alert);
        common_alert.kafka_topic = kafka_message.topic.clone();
        common_alert.kafka_partition = kafka_message.partition as u32;
        common_alert.kafka_offset = kafka_message.offset as u64;

        // Create NGAV specific alert
        let mut ngav_row = NgavAlertRow::from(ngav_alert);
        ngav_row.kafka_topic = kafka_message.topic.clone();
        ngav_row.kafka_partition = kafka_message.partition as u32;
        ngav_row.kafka_offset = kafka_message.offset as u64;

        // Store both alerts
        clickhouse.insert_common_alert(&common_alert).await?;
        clickhouse.insert_ngav_alert(&ngav_row).await?;

        info!(
            "Stored NGAV alert: {} (severity: {})",
            ngav_alert.get_alert_key(),
            ngav_alert.get_severity_level()
        );

        Ok(())
    }

    pub async fn get_consumer_status(&self) -> HashMap<String, serde_json::Value> {
        let consumers = self.consumers.read().await;
        let data_mapping = self.data_source_mapping.read().await;
        
        let mut status = HashMap::new();
        
        for (config_id, instance) in consumers.iter() {
            let data_type = data_mapping.get(config_id)
                .map(|s| s.as_str())
                .unwrap_or("unknown");
            
            status.insert(
                format!("{}_{}", config_id, instance.topic),
                serde_json::json!({
                    "config_id": config_id,
                    "topic": instance.topic,
                    "data_type": data_type,
                    "status": "running"
                })
            );
        }
        
        status
    }
}

impl Drop for ConsumerService {
    fn drop(&mut self) {
        // This will be called when the service is dropped
        info!("ConsumerService dropped");
    }
}