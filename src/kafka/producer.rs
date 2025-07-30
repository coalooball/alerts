use anyhow::Result;
use log::{error, info};
use rdkafka::{
    config::{ClientConfig, FromClientConfig},
    producer::{FutureProducer, FutureRecord},
};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Duration;

use crate::alert::AlertMessage;
use crate::edr_alert::EdrAlert;
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

    pub async fn send_edr_alert(&self, alert: EdrAlert) -> Result<()> {
        let json = serde_json::to_string(&alert)?;
        let alert_key = alert.get_alert_key();
        let record = FutureRecord::to(&self.config.topic)
            .payload(json.as_bytes())
            .key(&alert_key);

        match self.producer.send(record, Duration::from_secs(5)).await {
            Ok((partition, offset)) => {
                info!(
                    "EDR Alert sent successfully to partition {} at offset {} [{}]",
                    partition, offset, alert.report_name
                );
                Ok(())
            }
            Err((e, _)) => {
                error!("Failed to send EDR alert: {}", e);
                Err(anyhow::anyhow!("Failed to send EDR alert: {}", e))
            }
        }
    }

    pub async fn send_edr_batch(&self, alerts: Vec<EdrAlert>) -> Result<()> {
        for alert in alerts {
            if let Err(e) = self.send_edr_alert(alert).await {
                error!("Failed to send EDR alert in batch: {}", e);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        Ok(())
    }

    pub async fn load_and_send_jsonl_file(&self, file_path: &str) -> Result<usize> {
        info!("Loading EDR alerts from: {}", file_path);
        
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let mut sent_count = 0;
        let mut batch = Vec::new();
        const BATCH_SIZE: usize = 50;

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<EdrAlert>(&line) {
                Ok(alert) => {
                    batch.push(alert);
                    
                    if batch.len() >= BATCH_SIZE {
                        self.send_edr_batch(batch).await?;
                        sent_count += BATCH_SIZE;
                        batch = Vec::new();
                        info!("Sent batch of {} alerts, total: {}", BATCH_SIZE, sent_count);
                    }
                }
                Err(e) => {
                    error!("Failed to parse EDR alert: {}", e);
                }
            }
        }

        if !batch.is_empty() {
            let remaining = batch.len();
            self.send_edr_batch(batch).await?;
            sent_count += remaining;
            info!("Sent final batch of {} alerts", remaining);
        }

        info!("Successfully sent {} EDR alerts from {}", sent_count, file_path);
        Ok(sent_count)
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

    #[test]
    fn test_edr_alert_parsing() -> Result<()> {
        let json_data = r#"{"schema":1,"create_time":"2022-07-19T17:48:25.018Z","device_external_ip":"130.126.255.183","device_id":98483951,"device_internal_ip":"192.168.223.128","device_name":"WIN-32-H1","device_os":"WINDOWS","ioc_hit":"test ioc","ioc_id":"565644-0","org_key":"7DMF69PK","parent_cmdline":"C:\\Windows\\Explorer.EXE","parent_guid":"7DMF69PK-test","parent_hash":["hash1","hash2"],"parent_path":"c:\\windows\\explorer.exe","parent_pid":1544,"parent_publisher":[{"name":"Microsoft Windows","state":"SIGNED"}],"parent_reputation":"REP_WHITE","parent_username":"WIN-32-H1\\user","process_cmdline":"cmd.exe","process_guid":"7DMF69PK-test-process","process_hash":["hash3","hash4"],"process_path":"c:\\windows\\system32\\cmd.exe","process_pid":2872,"process_publisher":[{"name":"Microsoft Windows","state":"SIGNED"}],"process_reputation":"REP_WHITE","process_username":"WIN-32-H1\\user","report_id":"test-report-123","report_name":"Test Alert","report_tags":["attack","test"],"severity":1,"type":"watchlist.hit","watchlists":[{"id":"test-id","name":"Test Watchlist"}]}"#;

        let edr_alert: crate::edr_alert::EdrAlert = serde_json::from_str(json_data)?;
        
        assert_eq!(edr_alert.device_name, "WIN-32-H1");
        assert_eq!(edr_alert.severity, 1);
        assert_eq!(edr_alert.get_severity_level(), "critical");
        assert_eq!(edr_alert.report_name, "Test Alert");
        assert!(edr_alert.is_critical());
        assert!(edr_alert.contains_tag("attack"));
        
        let alert_key = edr_alert.get_alert_key();
        assert_eq!(alert_key, "WIN-32-H1_test-report-123");
        
        Ok(())
    }
}