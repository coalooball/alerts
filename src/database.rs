use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::{env, fs};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "10.26.64.224".to_string(),
            port: 5432,
            database: "alert_server".to_string(),
            username: "postgres".to_string(),
            password: "postgres".to_string(),
        }
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("DB_HOST").unwrap_or_else(|_| "10.26.64.224".to_string()),
            port: env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            database: env::var("DB_NAME").unwrap_or_else(|_| "alert_server".to_string()),
            username: env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string()),
            password: env::var("DB_PASSWORD").unwrap_or_else(|_| "postgres".to_string()),
        }
    }

    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct KafkaConfigRow {
    pub id: uuid::Uuid,
    pub name: String,
    pub bootstrap_servers: String,
    pub topic: String,
    pub group_id: String,
    pub message_timeout_ms: i32,
    pub request_timeout_ms: i32,
    pub retry_backoff_ms: i32,
    pub retries: i32,
    pub auto_offset_reset: String,
    pub enable_auto_commit: bool,
    pub auto_commit_interval_ms: i32,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let pool = PgPool::connect(&config.connection_string()).await?;
        Ok(Self { pool })
    }

    pub async fn initialize_schema(&self) -> Result<()> {
        log::info!("🔧 Initializing database schema...");

        // Check if tables already exist
        let table_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'kafka_configs')"
        )
        .fetch_one(&self.pool)
        .await?;

        if table_exists {
            log::info!("✅ Database schema already exists, skipping initialization");
            return Ok(());
        }

        // Read and execute init.sql
        let init_sql_path = "init.sql";
        let sql_content = fs::read_to_string(init_sql_path)
            .map_err(|e| anyhow::anyhow!("Could not read init.sql file: {}. Please ensure init.sql exists in the project root.", e))?;

        log::info!("📄 Reading initialization script from {}", init_sql_path);
        
        // Split SQL content by statements and execute each one
        // Note: This is a simple approach that works for most cases
        let statements: Vec<&str> = sql_content
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with("--"))
            .collect();

        for statement in statements {
            if !statement.trim().is_empty() {
                log::debug!("Executing SQL: {}", statement);
                sqlx::query(statement)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to execute SQL statement: {} - Error: {}", statement, e))?;
            }
        }

        log::info!("✅ Database schema initialized from init.sql");

        Ok(())
    }

    pub async fn get_active_kafka_config(&self) -> Result<Option<KafkaConfigRow>> {
        let row = sqlx::query_as::<_, KafkaConfigRow>(
            "SELECT * FROM kafka_configs WHERE is_active = true LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_kafka_config_by_name(&self, name: &str) -> Result<Option<KafkaConfigRow>> {
        let row = sqlx::query_as::<_, KafkaConfigRow>(
            "SELECT * FROM kafka_configs WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn create_kafka_config(&self, config: &KafkaConfigRow) -> Result<uuid::Uuid> {
        let id = uuid::Uuid::new_v4();
        let now = chrono::Utc::now();

        sqlx::query(
            r#"
            INSERT INTO kafka_configs (
                id, name, bootstrap_servers, topic, group_id,
                message_timeout_ms, request_timeout_ms, retry_backoff_ms, retries,
                auto_offset_reset, enable_auto_commit, auto_commit_interval_ms,
                is_active, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#
        )
        .bind(&id)
        .bind(&config.name)
        .bind(&config.bootstrap_servers)
        .bind(&config.topic)
        .bind(&config.group_id)
        .bind(&config.message_timeout_ms)
        .bind(&config.request_timeout_ms)
        .bind(&config.retry_backoff_ms)
        .bind(&config.retries)
        .bind(&config.auto_offset_reset)
        .bind(&config.enable_auto_commit)
        .bind(&config.auto_commit_interval_ms)
        .bind(&config.is_active)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn update_kafka_config(&self, id: uuid::Uuid, config: &KafkaConfigRow) -> Result<()> {
        let now = chrono::Utc::now();

        sqlx::query(
            r#"
            UPDATE kafka_configs SET
                name = $2, bootstrap_servers = $3, topic = $4, group_id = $5,
                message_timeout_ms = $6, request_timeout_ms = $7, retry_backoff_ms = $8, retries = $9,
                auto_offset_reset = $10, enable_auto_commit = $11, auto_commit_interval_ms = $12,
                is_active = $13, updated_at = $14
            WHERE id = $1
            "#
        )
        .bind(&id)
        .bind(&config.name)
        .bind(&config.bootstrap_servers)
        .bind(&config.topic)
        .bind(&config.group_id)
        .bind(&config.message_timeout_ms)
        .bind(&config.request_timeout_ms)
        .bind(&config.retry_backoff_ms)
        .bind(&config.retries)
        .bind(&config.auto_offset_reset)
        .bind(&config.enable_auto_commit)
        .bind(&config.auto_commit_interval_ms)
        .bind(&config.is_active)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn set_active_kafka_config(&self, id: uuid::Uuid) -> Result<()> {
        // First, set all configs to inactive
        sqlx::query("UPDATE kafka_configs SET is_active = false")
            .execute(&self.pool)
            .await?;

        // Then set the specified config to active
        sqlx::query("UPDATE kafka_configs SET is_active = true WHERE id = $1")
            .bind(&id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn list_kafka_configs(&self) -> Result<Vec<KafkaConfigRow>> {
        let rows = sqlx::query_as::<_, KafkaConfigRow>(
            "SELECT * FROM kafka_configs ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let row = sqlx::query("SELECT 1 as test")
            .fetch_one(&self.pool)
            .await?;

        let test_value: i32 = row.get("test");
        Ok(test_value == 1)
    }
}