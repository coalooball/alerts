use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use crate::database::{Database, DatabaseConfig, KafkaConfigRow};

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

    pub async fn from_database() -> Result<Self> {
        let db_config = DatabaseConfig::from_env();
        let database = Database::new(db_config).await?;
        
        if let Some(config_row) = database.get_active_kafka_config().await? {
            Ok(Self::from_database_row(&config_row))
        } else {
            // If no active config found, return default
            Ok(Self::default())
        }
    }

    pub async fn from_database_by_name(name: &str) -> Result<Option<Self>> {
        let db_config = DatabaseConfig::from_env();
        let database = Database::new(db_config).await?;
        
        if let Some(config_row) = database.get_kafka_config_by_name(name).await? {
            Ok(Some(Self::from_database_row(&config_row)))
        } else {
            Ok(None)
        }
    }

    fn from_database_row(row: &KafkaConfigRow) -> Self {
        Self {
            bootstrap_servers: row.bootstrap_servers.clone(),
            topic: row.topic.clone(),
            group_id: row.group_id.clone(),
            producer: KafkaProducerConfig {
                message_timeout_ms: row.message_timeout_ms as u32,
                request_timeout_ms: row.request_timeout_ms as u32,
                retry_backoff_ms: row.retry_backoff_ms as u32,
                retries: row.retries as u32,
            },
            consumer: KafkaConsumerConfig {
                auto_offset_reset: row.auto_offset_reset.clone(),
                enable_auto_commit: row.enable_auto_commit,
                auto_commit_interval_ms: row.auto_commit_interval_ms as u32,
            },
        }
    }

    pub fn to_database_row(&self, name: String, is_active: bool) -> KafkaConfigRow {
        let now = chrono::Utc::now();
        KafkaConfigRow {
            id: uuid::Uuid::new_v4(),
            name,
            bootstrap_servers: self.bootstrap_servers.clone(),
            topic: self.topic.clone(),
            group_id: self.group_id.clone(),
            message_timeout_ms: self.producer.message_timeout_ms as i32,
            request_timeout_ms: self.producer.request_timeout_ms as i32,
            retry_backoff_ms: self.producer.retry_backoff_ms as i32,
            retries: self.producer.retries as i32,
            auto_offset_reset: self.consumer.auto_offset_reset.clone(),
            enable_auto_commit: self.consumer.enable_auto_commit,
            auto_commit_interval_ms: self.consumer.auto_commit_interval_ms as i32,
            is_active,
            created_at: now,
            updated_at: now,
        }
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