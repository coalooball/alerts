use anyhow::Result;
use log::{error, info, warn};
use rdkafka::{
    config::{ClientConfig, FromClientConfig},
    consumer::{Consumer, StreamConsumer},
    Message,
};

use crate::alert::AlertMessage;
use crate::edr_alert::EdrAlert;
use super::config::KafkaConfig;

pub struct KafkaConsumer {
    consumer: StreamConsumer,
    config: KafkaConfig,
}

impl KafkaConsumer {
    pub async fn new(config: KafkaConfig) -> Result<Self> {
        let mut consumer_config = ClientConfig::new();
        consumer_config
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("group.id", &config.group_id)
            .set("auto.offset.reset", &config.consumer.auto_offset_reset)
            .set("enable.auto.commit", &config.consumer.enable_auto_commit.to_string())
            .set("auto.commit.interval.ms", &config.consumer.auto_commit_interval_ms.to_string());

        let consumer = StreamConsumer::from_config(&consumer_config)?;

        Ok(KafkaConsumer { consumer, config })
    }

    pub async fn consume_alerts(&self) -> Result<()> {
        self.consumer.subscribe(&[&self.config.topic])?;

        info!("Starting to consume alerts from topic: {}", self.config.topic);

        loop {
            match self.consumer.recv().await {
                Ok(msg) => {
                    let payload = match msg.payload_view::<str>() {
                        Some(Ok(s)) => s,
                        Some(Err(e)) => {
                            error!("Error deserializing message payload: {}", e);
                            continue;
                        }
                        None => {
                            warn!("Empty message received");
                            continue;
                        }
                    };

                    if let Ok(alert) = serde_json::from_str::<AlertMessage>(payload) {
                        info!("Received simple alert: {:?}", alert);
                        self.process_alert(alert).await;
                    } else if let Ok(edr_alert) = serde_json::from_str::<EdrAlert>(payload) {
                        info!("Received EDR alert: {} - {}", edr_alert.report_name, edr_alert.device_name);
                        self.process_edr_alert(edr_alert).await;
                    } else {
                        error!("Error deserializing message - unknown format");
                    }
                }
                Err(e) => {
                    error!("Error receiving message: {}", e);
                    if e.to_string().contains("timed out") {
                        continue;
                    }
                    break Ok(());
                }
            }
        }
    }

    async fn process_alert(&self, alert: AlertMessage) {
        match alert.level.as_str() {
            "critical" => {
                error!("🚨 CRITICAL ALERT: {}", alert.message);
            }
            "warning" => {
                warn!("⚠️  WARNING: {}", alert.message);
            }
            "info" => {
                info!("ℹ️  INFO: {}", alert.message);
            }
            _ => {
                warn!("❓ Unknown alert level: {}", alert.level);
            }
        }
    }

    async fn process_edr_alert(&self, alert: EdrAlert) {
        let severity_level = alert.get_severity_level();
        let process_info = alert.get_process_info();
        
        match severity_level {
            "critical" => {
                error!("🚨 CRITICAL EDR ALERT: {} | Device: {} | Process: {}", 
                       alert.report_name, alert.device_name, process_info);
                
                if alert.contains_tag("attack") {
                    error!("🔴 ATTACK DETECTED: Contains attack-related tags");
                }
            }
            "high" => {
                error!("🔴 HIGH SEVERITY EDR ALERT: {} | Device: {} | IOC: {}", 
                       alert.report_name, alert.device_name, alert.ioc_hit);
            }
            "medium" => {
                warn!("🟡 MEDIUM SEVERITY EDR ALERT: {} | Device: {} | User: {}", 
                      alert.report_name, alert.device_name, alert.process_username);
            }
            "low" => {
                info!("🟢 LOW SEVERITY EDR ALERT: {} | Device: {} | Tags: {:?}", 
                      alert.report_name, alert.device_name, alert.report_tags);
            }
            _ => {
                warn!("❓ UNKNOWN SEVERITY EDR ALERT: {} | Severity: {}", 
                      alert.report_name, alert.severity);
            }
        }

        if alert.is_critical() {
            info!("📧 Critical alert - triggering notifications for report: {}", alert.report_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kafka::config::{KafkaConfig, KafkaProducerConfig, KafkaConsumerConfig};
    use rdkafka::config::ClientConfig;

    #[tokio::test]
    async fn test_kafka_consumer_creation() {
        let config = KafkaConfig {
            bootstrap_servers: "localhost:9092".to_string(),
            topic: "test-alerts".to_string(),
            group_id: "test-consumer-group".to_string(),
            producer: KafkaProducerConfig {
                message_timeout_ms: 5000,
                request_timeout_ms: 5000,
                retry_backoff_ms: 100,
                retries: 3,
            },
            consumer: KafkaConsumerConfig {
                auto_offset_reset: "latest".to_string(),
                enable_auto_commit: false,
                auto_commit_interval_ms: 2000,
            },
        };

        let consumer_result = KafkaConsumer::new(config).await;
        assert!(consumer_result.is_ok());
    }

    #[tokio::test]
    async fn test_kafka_consumer_with_config_file() -> Result<()> {
        let config = KafkaConfig::from_file("config.toml")
            .unwrap_or_else(|_| KafkaConfig::default());
        
        let consumer = KafkaConsumer::new(config.clone()).await;
        assert!(consumer.is_ok());
        
        let consumer = consumer.unwrap();
        assert_eq!(consumer.config.bootstrap_servers, config.bootstrap_servers);
        assert_eq!(consumer.config.topic, config.topic);
        assert_eq!(consumer.config.group_id, config.group_id);
        
        Ok(())
    }

    #[test]
    fn test_consumer_config_setup() {
        let config = KafkaConfig::default();
        
        let mut consumer_config = ClientConfig::new();
        consumer_config
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("group.id", &config.group_id)
            .set("auto.offset.reset", &config.consumer.auto_offset_reset)
            .set("enable.auto.commit", &config.consumer.enable_auto_commit.to_string())
            .set("auto.commit.interval.ms", &config.consumer.auto_commit_interval_ms.to_string());

        let bootstrap_servers = consumer_config.get("bootstrap.servers").unwrap();
        assert_eq!(bootstrap_servers, "localhost:9092");
        
        let group_id = consumer_config.get("group.id").unwrap();
        assert_eq!(group_id, "alerts-consumer-group");
        
        let auto_offset_reset = consumer_config.get("auto.offset.reset").unwrap();
        assert_eq!(auto_offset_reset, "earliest");
    }

    #[tokio::test]
    async fn test_process_alert_levels() {
        let config = KafkaConfig::default();
        let consumer = KafkaConsumer::new(config).await.unwrap();

        let critical_alert = AlertMessage::critical(
            "test-001".to_string(),
            "Critical test message".to_string(),
        );
        consumer.process_alert(critical_alert).await;

        let warning_alert = AlertMessage::warning(
            "test-002".to_string(),
            "Warning test message".to_string(),
        );
        consumer.process_alert(warning_alert).await;

        let info_alert = AlertMessage::info(
            "test-003".to_string(),
            "Info test message".to_string(),
        );
        consumer.process_alert(info_alert).await;
    }

    #[test]
    fn test_alert_message_deserialization() -> Result<()> {
        let json_alert = r#"{"id":"test-001","level":"critical","message":"Test message","timestamp":1234567890}"#;
        
        let alert: AlertMessage = serde_json::from_str(json_alert)?;
        
        assert_eq!(alert.id, "test-001");
        assert_eq!(alert.level, "critical");
        assert_eq!(alert.message, "Test message");
        assert_eq!(alert.timestamp, 1234567890);
        
        Ok(())
    }

    #[test]
    fn test_invalid_alert_deserialization() {
        let invalid_json = r#"{"invalid":"data"}"#;
        
        let result: Result<AlertMessage, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_consumer_subscription_topic() -> Result<()> {
        let config = KafkaConfig {
            bootstrap_servers: "localhost:9092".to_string(),
            topic: "custom-alerts-topic".to_string(),
            group_id: "custom-group".to_string(),
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

        let consumer = KafkaConsumer::new(config.clone()).await?;
        assert_eq!(consumer.config.topic, "custom-alerts-topic");
        assert_eq!(consumer.config.group_id, "custom-group");
        
        Ok(())
    }
}