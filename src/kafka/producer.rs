use anyhow::Result;
use log::{error, info};
use rdkafka::{
    config::{ClientConfig, FromClientConfig},
    producer::{FutureProducer, FutureRecord},
};
use std::time::Duration;

use crate::alert::AlertMessage;
use super::config::KafkaConfig;

pub struct KafkaProducer {
    producer: FutureProducer,
    config: KafkaConfig,
}

impl KafkaProducer {
    pub async fn new(config: KafkaConfig) -> Result<Self> {
        let mut producer_config = ClientConfig::new();
        producer_config
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("message.timeout.ms", &config.producer.message_timeout_ms.to_string())
            .set("request.timeout.ms", &config.producer.request_timeout_ms.to_string())
            .set("retry.backoff.ms", &config.producer.retry_backoff_ms.to_string())
            .set("retries", &config.producer.retries.to_string());

        let producer = FutureProducer::from_config(&producer_config)?;

        Ok(KafkaProducer { producer, config })
    }

    pub async fn send_alert(&self, alert: AlertMessage) -> Result<()> {
        let json = serde_json::to_string(&alert)?;
        let record = FutureRecord::to(&self.config.topic)
            .payload(json.as_bytes())
            .key(&alert.id);

        match self.producer.send(record, Duration::from_secs(5)).await {
            Ok((partition, offset)) => {
                info!(
                    "Alert sent successfully to partition {} at offset {}",
                    partition, offset
                );
                Ok(())
            }
            Err((e, _)) => {
                error!("Failed to send alert: {}", e);
                Err(anyhow::anyhow!("Failed to send alert: {}", e))
            }
        }
    }

    pub async fn send_batch(&self, alerts: Vec<AlertMessage>) -> Result<()> {
        for alert in alerts {
            if let Err(e) = self.send_alert(alert).await {
                error!("Failed to send alert in batch: {}", e);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kafka::config::{KafkaConfig, KafkaProducerConfig, KafkaConsumerConfig};
    use rdkafka::config::ClientConfig;

    #[tokio::test]
    async fn test_kafka_producer_creation() {
        let config = KafkaConfig {
            bootstrap_servers: "localhost:9092".to_string(),
            topic: "test-alerts".to_string(),
            group_id: "test-group".to_string(),
            producer: KafkaProducerConfig {
                message_timeout_ms: 3000,
                request_timeout_ms: 3000,
                retry_backoff_ms: 50,
                retries: 2,
            },
            consumer: KafkaConsumerConfig {
                auto_offset_reset: "earliest".to_string(),
                enable_auto_commit: true,
                auto_commit_interval_ms: 1000,
            },
        };

        let producer_result = KafkaProducer::new(config).await;
        assert!(producer_result.is_ok());
    }

    #[tokio::test]
    async fn test_kafka_producer_with_config_file() -> Result<()> {
        let config = KafkaConfig::from_file("config.toml")
            .unwrap_or_else(|_| KafkaConfig::default());
        
        let producer = KafkaProducer::new(config.clone()).await;
        assert!(producer.is_ok());
        
        let producer = producer.unwrap();
        assert_eq!(producer.config.bootstrap_servers, config.bootstrap_servers);
        assert_eq!(producer.config.topic, config.topic);
        
        Ok(())
    }

    #[test]
    fn test_client_config_setup() {
        let config = KafkaConfig::default();
        
        let mut producer_config = ClientConfig::new();
        producer_config
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("message.timeout.ms", &config.producer.message_timeout_ms.to_string())
            .set("request.timeout.ms", &config.producer.request_timeout_ms.to_string())
            .set("retry.backoff.ms", &config.producer.retry_backoff_ms.to_string())
            .set("retries", &config.producer.retries.to_string());

        let bootstrap_servers = producer_config.get("bootstrap.servers").unwrap();
        assert_eq!(bootstrap_servers, "localhost:9092");
        
        let timeout = producer_config.get("message.timeout.ms").unwrap();
        assert_eq!(timeout, "5000");
    }

    #[tokio::test]
    async fn test_send_alert_message_format() -> Result<()> {
        let alert = AlertMessage::critical(
            "test-001".to_string(),
            "Test critical alert".to_string(),
        );
        
        let json = serde_json::to_string(&alert)?;
        let parsed: AlertMessage = serde_json::from_str(&json)?;
        
        assert_eq!(parsed.id, "test-001");
        assert_eq!(parsed.level, "critical");
        assert_eq!(parsed.message, "Test critical alert");
        assert!(parsed.timestamp > 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_batch_alerts_creation() {
        let alerts = vec![
            AlertMessage::critical("alert-001".to_string(), "Critical issue".to_string()),
            AlertMessage::warning("alert-002".to_string(), "Warning issue".to_string()),
            AlertMessage::info("alert-003".to_string(), "Info message".to_string()),
        ];
        
        assert_eq!(alerts.len(), 3);
        assert_eq!(alerts[0].level, "critical");
        assert_eq!(alerts[1].level, "warning");
        assert_eq!(alerts[2].level, "info");
    }
}