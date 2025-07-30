use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KafkaProducerConfig {
    pub message_timeout_ms: u32,
    pub request_timeout_ms: u32,
    pub retry_backoff_ms: u32,
    pub retries: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KafkaConsumerConfig {
    pub auto_offset_reset: String,
    pub enable_auto_commit: bool,
    pub auto_commit_interval_ms: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KafkaConfig {
    pub bootstrap_servers: String,
    pub topic: String,
    pub group_id: String,
    pub producer: KafkaProducerConfig,
    pub consumer: KafkaConsumerConfig,
}

#[derive(Debug, Deserialize)]
struct Config {
    kafka: KafkaConfig,
}

impl KafkaConfig {
    pub fn from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config.kafka)
    }
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: "localhost:9092".to_string(),
            topic: "alerts".to_string(),
            group_id: "alerts-consumer-group".to_string(),
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_kafka_config_default() {
        let config = KafkaConfig::default();
        
        assert_eq!(config.bootstrap_servers, "localhost:9092");
        assert_eq!(config.topic, "alerts");
        assert_eq!(config.group_id, "alerts-consumer-group");
        assert_eq!(config.producer.message_timeout_ms, 5000);
        assert_eq!(config.consumer.auto_offset_reset, "earliest");
        assert!(config.consumer.enable_auto_commit);
    }

    #[test]
    fn test_kafka_config_from_file() -> Result<()> {
        let toml_content = r#"
[kafka]
bootstrap_servers = "test-server:9092"
topic = "test-topic"
group_id = "test-group"

[kafka.producer]
message_timeout_ms = 3000
request_timeout_ms = 3000
retry_backoff_ms = 50
retries = 5

[kafka.consumer]
auto_offset_reset = "latest"
enable_auto_commit = false
auto_commit_interval_ms = 2000
"#;

        let temp_file = NamedTempFile::new()?;
        fs::write(temp_file.path(), toml_content)?;
        
        let config = KafkaConfig::from_file(temp_file.path().to_str().unwrap())?;
        
        assert_eq!(config.bootstrap_servers, "test-server:9092");
        assert_eq!(config.topic, "test-topic");
        assert_eq!(config.group_id, "test-group");
        assert_eq!(config.producer.message_timeout_ms, 3000);
        assert_eq!(config.producer.retries, 5);
        assert_eq!(config.consumer.auto_offset_reset, "latest");
        assert!(!config.consumer.enable_auto_commit);
        assert_eq!(config.consumer.auto_commit_interval_ms, 2000);
        
        Ok(())
    }

    #[test]
    fn test_kafka_config_file_not_found() {
        let result = KafkaConfig::from_file("nonexistent_file.toml");
        assert!(result.is_err());
    }

    #[test]  
    fn test_kafka_config_invalid_toml() -> Result<()> {
        let invalid_toml = "invalid toml content [[[";
        
        let temp_file = NamedTempFile::new()?;
        fs::write(temp_file.path(), invalid_toml)?;
        
        let result = KafkaConfig::from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
        
        Ok(())
    }
}