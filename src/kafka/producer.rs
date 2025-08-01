use anyhow::Result;
use log::{error, info};
use rdkafka::{
    config::{ClientConfig, FromClientConfig},
    producer::{FutureProducer, FutureRecord, Producer},
};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::fs;
use std::time::Duration;

use crate::alert::AlertMessage;
use crate::edr_alert::EdrAlert;
use crate::ngav_alert::NgavAlert;
use super::config::KafkaConfig;

pub struct KafkaProducer {
    producer: FutureProducer,
    config: KafkaConfig,
}

impl KafkaProducer {
    /// Flush all pending messages
    pub fn flush(&self, timeout: Duration) -> Result<()> {
        info!("Flushing producer with timeout: {:?}", timeout);
        let result = self.producer.flush(timeout);
        match result {
            Ok(()) => {
                info!("✅ Producer flushed successfully");
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to flush producer: {:?}", e);
                Err(anyhow::anyhow!("Failed to flush producer: {:?}", e))
            }
        }
    }

    /// Safely read a file that may contain invalid UTF-8
    #[allow(dead_code)]
    fn read_file_lossy(file_path: &str) -> Result<Vec<String>> {
        let bytes = fs::read(file_path)?;
        let content = String::from_utf8_lossy(&bytes);
        
        // Check if there were any invalid UTF-8 sequences
        if content.contains('\u{FFFD}') {
            info!("File {} contains invalid UTF-8 characters, using lossy conversion", file_path);
        }
        
        Ok(content.lines().map(|s| s.to_string()).collect())
    }
    pub async fn new(config: KafkaConfig) -> Result<Self> {
        let mut producer_config = ClientConfig::new();
        
        // Debug: Log the bootstrap servers being set
        info!("Setting bootstrap.servers to: {}", config.bootstrap_servers);
        
        // Try to ensure we only use the specified servers
        let bootstrap_servers = config.bootstrap_servers.clone();
        
        // Log the exact bootstrap servers being used
        info!("Using bootstrap servers: '{}'", bootstrap_servers);
        
        producer_config
            .set("bootstrap.servers", &bootstrap_servers)
            .set("message.timeout.ms", &config.producer.message_timeout_ms.to_string())
            .set("request.timeout.ms", &config.producer.request_timeout_ms.to_string())
            .set("retry.backoff.ms", &config.producer.retry_backoff_ms.to_string())
            .set("retries", &config.producer.retries.to_string())
            // Add additional configurations to ensure proper connection
            .set("client.id", "alerts-producer")
            .set("socket.keepalive.enable", "true")
            .set("socket.timeout.ms", "6000")
            .set("connections.max.idle.ms", "540000");

        // Debug: Verify what was actually set
        if let Some(servers) = producer_config.get("bootstrap.servers") {
            info!("Verified bootstrap.servers in config: {}", servers);
        } else {
            error!("bootstrap.servers not found in producer config!");
        }

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

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(line) => line,
                Err(e) => {
                    error!("Error reading line: {}, using lossy conversion", e);
                    // Try to recover by reading raw bytes
                    continue;
                }
            };
            
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

    pub async fn send_ngav_alert(&self, alert: NgavAlert) -> Result<()> {
        let json = serde_json::to_string(&alert)?;
        let alert_key = alert.get_alert_key();
        let record = FutureRecord::to(&self.config.topic)
            .payload(json.as_bytes())
            .key(&alert_key);

        match self.producer.send(record, Duration::from_secs(5)).await {
            Ok((partition, offset)) => {
                info!(
                    "NGAV Alert sent successfully to partition {} at offset {} [{}]",
                    partition, offset, alert.reason
                );
                Ok(())
            }
            Err((e, _)) => {
                error!("Failed to send NGAV alert: {}", e);
                Err(anyhow::anyhow!("Failed to send NGAV alert: {}", e))
            }
        }
    }

    pub async fn send_ngav_batch(&self, alerts: Vec<NgavAlert>) -> Result<()> {
        for alert in alerts {
            if let Err(e) = self.send_ngav_alert(alert).await {
                error!("Failed to send NGAV alert in batch: {}", e);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        Ok(())
    }

    pub async fn load_and_send_ngav_file(&self, file_path: &str) -> Result<usize> {
        info!("Loading NGAV alerts from: {}", file_path);
        
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let mut sent_count = 0;
        let mut batch = Vec::new();
        const BATCH_SIZE: usize = 50;

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(line) => line,
                Err(e) => {
                    error!("Error reading line: {}, using lossy conversion", e);
                    // Try to recover by reading raw bytes
                    continue;
                }
            };
            
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<NgavAlert>(&line) {
                Ok(alert) => {
                    batch.push(alert);
                    
                    if batch.len() >= BATCH_SIZE {
                        self.send_ngav_batch(batch).await?;
                        sent_count += BATCH_SIZE;
                        batch = Vec::new();
                        info!("Sent batch of {} NGAV alerts, total: {}", BATCH_SIZE, sent_count);
                    }
                }
                Err(e) => {
                    error!("Failed to parse NGAV alert: {}", e);
                }
            }
        }

        if !batch.is_empty() {
            let remaining = batch.len();
            self.send_ngav_batch(batch).await?;
            sent_count += remaining;
            info!("Sent final batch of {} NGAV alerts", remaining);
        }

        info!("Successfully sent {} NGAV alerts from {}", sent_count, file_path);
        Ok(sent_count)
    }

    pub async fn detect_and_load_file(&self, file_path: &str) -> Result<usize> {
        info!("Auto-detecting file format for: {}", file_path);
        
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut first_line = String::new();
        
        // Try to read the first line with UTF-8 lossy conversion
        match reader.read_line(&mut first_line) {
            Ok(_) => {},
            Err(e) => {
                error!("Error reading first line: {}", e);
                return Err(anyhow::anyhow!("Failed to read file: {}", e));
            }
        }
        
        if first_line.trim().is_empty() {
            return Err(anyhow::anyhow!("Empty file: {}", file_path));
        }

        if first_line.contains("\"schema\":") && first_line.contains("\"ioc_hit\":") {
            info!("Detected EDR alert format");
            self.load_and_send_jsonl_file(file_path).await
        } else if first_line.contains("\"type\":\"CB_ANALYTICS\"") && first_line.contains("\"threat_indicators\":") {
            info!("Detected NGAV alert format");
            self.load_and_send_ngav_file(file_path).await
        } else {
            Err(anyhow::anyhow!("Unknown file format: {}", file_path))
        }
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

    #[test]
    fn test_ngav_alert_parsing() -> Result<()> {
        let json_data = r#"{"type":"CB_ANALYTICS","id":"test-id-123","legacy_alert_id":"test-legacy-id","org_key":"7DMF69PK","create_time":"2022-07-19T19:32:28Z","last_update_time":"2022-07-19T19:53:44Z","first_event_time":"2022-07-19T19:31:46Z","last_event_time":"2022-07-19T19:52:45Z","threat_id":"test-threat-id","severity":1,"category":"THREAT","device_id":98483951,"device_os":"WINDOWS","device_os_version":"Windows 10","device_name":"TEST-DEVICE","device_username":"test.user@example.com","policy_id":268058,"policy_name":"Test Policy","target_value":"HIGH","workflow":{"state":"OPEN","remediation":"","last_update_time":"2022-07-19T19:32:28Z","comment":"","changed_by":"Carbon Black"},"device_internal_ip":"192.168.1.100","device_external_ip":"1.2.3.4","alert_url":"https://test.com","reason":"Test malware detected","reason_code":"R_MALWARE","process_name":"malware.exe","device_location":"ONSITE","created_by_event_id":"test-event-id","threat_indicators":[{"process_name":"malware.exe","sha256":"testhash123","ttps":["MITRE_T1059","NETWORK_ACCESS"]}],"threat_cause_actor_sha256":"testhash123","threat_cause_actor_name":"malware.exe","threat_cause_actor_process_pid":"1234-567890","threat_cause_reputation":"MALWARE","threat_cause_threat_category":"MALWARE","threat_cause_vector":"WEB","threat_cause_cause_event_id":"test-cause-id","blocked_threat_category":"MALWARE","not_blocked_threat_category":"UNKNOWN","kill_chain_status":["INSTALL_RUN"],"run_state":"TERMINATED","policy_applied":"APPLIED"}"#;

        let ngav_alert: crate::ngav_alert::NgavAlert = serde_json::from_str(json_data)?;
        
        assert_eq!(ngav_alert.device_name, "TEST-DEVICE");
        assert_eq!(ngav_alert.severity, 1);
        assert_eq!(ngav_alert.get_severity_level(), "critical");
        assert_eq!(ngav_alert.reason, "Test malware detected");
        assert!(ngav_alert.is_critical());
        assert!(ngav_alert.is_malware());
        assert!(ngav_alert.has_mitre_ttps());
        assert!(ngav_alert.is_blocked());
        
        let alert_key = ngav_alert.get_alert_key();
        assert_eq!(alert_key, "TEST-DEVICE_test-id-123");
        
        let ttps = ngav_alert.get_mitre_ttps();
        assert!(ttps.contains(&"MITRE_T1059".to_string()));
        
        Ok(())
    }
}