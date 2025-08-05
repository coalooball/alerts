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
            host: "127.0.0.1".to_string(),
            port: 5433,
            database: "alert_server".to_string(),
            username: "postgres".to_string(),
            password: "postgres".to_string(),
        }
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("DB_PORT")
                .unwrap_or_else(|_| "5433".to_string())
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

    // 连接到默认数据库（通常是postgres）的连接字符串
    pub fn default_connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/postgres",
            self.username, self.password, self.host, self.port
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClickHouseConfigRow {
    pub id: uuid::Uuid,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub database_name: String,
    pub username: String,
    pub password: Option<String>,
    pub use_tls: bool,
    pub connection_timeout_ms: i32,
    pub request_timeout_ms: i32,
    pub max_connections: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new_with_init(config: DatabaseConfig) -> Result<Self> {
        // 连接到默认数据库以删除和重新创建目标数据库
        log::info!("🔄 Initializing database '{}' (drop and recreate)...", config.database);
        
        let default_pool = PgPool::connect(&config.default_connection_string()).await?;
        
        // 终止所有连接到目标数据库的会话
        log::info!("🔌 Terminating existing connections to database '{}'...", config.database);
        sqlx::query(&format!(
            "SELECT pg_terminate_backend(pg_stat_activity.pid) 
             FROM pg_stat_activity 
             WHERE pg_stat_activity.datname = '{}' 
             AND pid <> pg_backend_pid()", 
            config.database
        ))
        .execute(&default_pool)
        .await?;

        // 删除数据库（如果存在）
        log::info!("🗑️ Dropping database '{}' if it exists...", config.database);
        sqlx::query(&format!("DROP DATABASE IF EXISTS \"{}\"", config.database))
            .execute(&default_pool)
            .await?;

        // 创建新数据库
        log::info!("🏗️ Creating database '{}'...", config.database);
        sqlx::query(&format!("CREATE DATABASE \"{}\"", config.database))
            .execute(&default_pool)
            .await?;

        default_pool.close().await;

        // 连接到新创建的数据库
        log::info!("📊 Connecting to newly created database '{}'...", config.database);
        let pool = PgPool::connect(&config.connection_string()).await?;
        
        let database = Self { pool };
        
        // 初始化数据库架构
        log::info!("🏗️ Initializing database schema...");
        database.initialize_schema().await?;
        
        log::info!("✅ Database initialization completed successfully");
        Ok(database)
    }

    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        // 首先连接到默认数据库（postgres）来检查目标数据库是否存在
        log::info!("🔍 Checking if database '{}' exists...", config.database);
        
        let default_pool = PgPool::connect(&config.default_connection_string()).await?;
        
        // 检查目标数据库是否存在
        let db_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = $1)"
        )
        .bind(&config.database)
        .fetch_one(&default_pool)
        .await?;

        if !db_exists {
            log::info!("📝 Database '{}' does not exist, creating it...", config.database);
            
            // 创建数据库
            sqlx::query(&format!("CREATE DATABASE \"{}\"", config.database))
                .execute(&default_pool)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create database '{}': {}", config.database, e))?;
            
            log::info!("✅ Database '{}' created successfully", config.database);
        } else {
            log::info!("✅ Database '{}' already exists", config.database);
        }

        // 关闭默认连接池
        default_pool.close().await;

        // 连接到目标数据库
        log::info!("🔗 Connecting to database '{}'...", config.database);
        let pool = PgPool::connect(&config.connection_string()).await?;
        
        log::info!("✅ Successfully connected to database '{}'", config.database);
        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn initialize_schema(&self) -> Result<()> {
        log::info!("🔧 Initializing database schema...");

        // Check if tables already exist (check both kafka_configs and clickhouse_config)
        let kafka_table_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'kafka_configs')"
        )
        .fetch_one(&self.pool)
        .await?;

        let clickhouse_table_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'clickhouse_config')"
        )
        .fetch_one(&self.pool)
        .await?;

        let table_exists = kafka_table_exists && clickhouse_table_exists;

        if table_exists {
            log::info!("✅ Database schema already exists, skipping initialization");
            return Ok(());
        }

        // Read and execute init.sql
        let init_sql_path = "init.sql";
        let sql_content = fs::read_to_string(init_sql_path)
            .map_err(|e| anyhow::anyhow!("Could not read init.sql file: {}. Please ensure init.sql exists in the project root.", e))?;

        log::info!("📄 Reading initialization script from {}", init_sql_path);
        
        // Use raw connection to execute the entire SQL script
        let mut conn = self.pool.acquire().await?;
        
        log::debug!("Executing complete SQL script with raw connection");
        sqlx::raw_sql(&sql_content)
            .execute(&mut *conn)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute init.sql: {}", e))?;

        log::info!("✅ Database schema initialized from init.sql");

        Ok(())
    }

    pub async fn get_active_kafka_config(&self) -> Result<Option<KafkaConfigRow>> {
        let row = sqlx::query_as::<_, KafkaConfigRow>(
            "SELECT * FROM kafka_configs WHERE is_active = true ORDER BY created_at ASC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_active_kafka_configs(&self) -> Result<Vec<KafkaConfigRow>> {
        let rows = sqlx::query_as::<_, KafkaConfigRow>(
            "SELECT * FROM kafka_configs WHERE is_active = true ORDER BY created_at ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
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

    pub async fn toggle_kafka_config_active(&self, id: uuid::Uuid, is_active: bool) -> Result<()> {
        sqlx::query("UPDATE kafka_configs SET is_active = $1 WHERE id = $2")
            .bind(is_active)
            .bind(&id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_kafka_config(&self, id: uuid::Uuid) -> Result<()> {
        sqlx::query("DELETE FROM kafka_configs WHERE id = $1")
            .bind(&id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_kafka_config_by_id(&self, id: uuid::Uuid) -> Result<Option<KafkaConfigRow>> {
        let row = sqlx::query_as::<_, KafkaConfigRow>(
            "SELECT * FROM kafka_configs WHERE id = $1"
        )
        .bind(&id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
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

    // ClickHouse configuration methods
    pub async fn get_clickhouse_config(&self) -> Result<Option<ClickHouseConfigRow>> {
        let row = sqlx::query_as::<_, ClickHouseConfigRow>(
            "SELECT * FROM clickhouse_config ORDER BY created_at ASC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn create_or_update_clickhouse_config(&self, config: &ClickHouseConfigRow) -> Result<uuid::Uuid> {
        // First, check if any config exists
        let existing_config = self.get_clickhouse_config().await?;
        
        if let Some(existing) = existing_config {
            // Update existing config
            let now = chrono::Utc::now();
            
            sqlx::query(
                r#"
                UPDATE clickhouse_config SET
                    name = $2, host = $3, port = $4, database_name = $5,
                    username = $6, password = $7, use_tls = $8,
                    connection_timeout_ms = $9, request_timeout_ms = $10, max_connections = $11,
                    updated_at = $12
                WHERE id = $1
                "#
            )
            .bind(&existing.id)
            .bind(&config.name)
            .bind(&config.host)
            .bind(&config.port)
            .bind(&config.database_name)
            .bind(&config.username)
            .bind(&config.password)
            .bind(&config.use_tls)
            .bind(&config.connection_timeout_ms)
            .bind(&config.request_timeout_ms)
            .bind(&config.max_connections)
            .bind(&now)
            .execute(&self.pool)
            .await?;

            Ok(existing.id)
        } else {
            // Create new config
            let id = uuid::Uuid::new_v4();
            let now = chrono::Utc::now();

            sqlx::query(
                r#"
                INSERT INTO clickhouse_config (
                    id, name, host, port, database_name, username, password, use_tls,
                    connection_timeout_ms, request_timeout_ms, max_connections,
                    created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                "#
            )
            .bind(&id)
            .bind(&config.name)
            .bind(&config.host)
            .bind(&config.port)
            .bind(&config.database_name)
            .bind(&config.username)
            .bind(&config.password)
            .bind(&config.use_tls)
            .bind(&config.connection_timeout_ms)
            .bind(&config.request_timeout_ms)
            .bind(&config.max_connections)
            .bind(&now)
            .bind(&now)
            .execute(&self.pool)
            .await?;

            Ok(id)
        }
    }

    pub async fn delete_clickhouse_config(&self) -> Result<()> {
        sqlx::query("DELETE FROM clickhouse_config")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Data source configuration methods
    pub async fn get_data_source_configs(&self) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                dsc.id,
                dsc.data_type,
                dsc.kafka_config_id,
                dsc.created_at,
                dsc.updated_at,
                kc.id as kafka_id,
                kc.name as kafka_name,
                kc.bootstrap_servers,
                kc.topic,
                kc.group_id,
                kc.is_active as kafka_is_active
            FROM data_source_configs dsc
            JOIN kafka_configs kc ON dsc.kafka_config_id = kc.id
            ORDER BY dsc.data_type, kc.name
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut configs = Vec::new();
        for row in rows {
            let config = serde_json::json!({
                "id": row.get::<uuid::Uuid, _>("id"),
                "data_type": row.get::<String, _>("data_type"),
                "kafka_config_id": row.get::<uuid::Uuid, _>("kafka_config_id"),
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
                "updated_at": row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
                "kafka_config": {
                    "id": row.get::<uuid::Uuid, _>("kafka_id"),
                    "name": row.get::<String, _>("kafka_name"),
                    "bootstrap_servers": row.get::<String, _>("bootstrap_servers"),
                    "topic": row.get::<String, _>("topic"),
                    "group_id": row.get::<String, _>("group_id"),
                    "is_active": row.get::<bool, _>("kafka_is_active")
                }
            });
            configs.push(config);
        }

        Ok(configs)
    }

    pub async fn save_data_source_config(
        &self,
        data_type: &str,
        kafka_config_ids: &[uuid::Uuid],
    ) -> Result<()> {
        // Start a transaction
        let mut tx = self.pool.begin().await?;

        // First, delete existing configs for this data type
        sqlx::query("DELETE FROM data_source_configs WHERE data_type = $1")
            .bind(data_type)
            .execute(&mut *tx)
            .await?;

        // Insert new configurations
        for kafka_config_id in kafka_config_ids {
            sqlx::query("INSERT INTO data_source_configs (data_type, kafka_config_id) VALUES ($1, $2)")
                .bind(data_type)
                .bind(kafka_config_id)
                .execute(&mut *tx)
                .await?;
        }

        // Commit the transaction
        tx.commit().await?;

        Ok(())
    }

    pub async fn get_data_source_configs_by_type(&self, data_type: &str) -> Result<Vec<uuid::Uuid>> {
        let rows = sqlx::query("SELECT kafka_config_id FROM data_source_configs WHERE data_type = $1")
            .bind(data_type)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|row| row.get::<uuid::Uuid, _>("kafka_config_id")).collect())
    }

    pub async fn delete_data_source_config(&self, data_type: &str) -> Result<()> {
        sqlx::query("DELETE FROM data_source_configs WHERE data_type = $1")
            .bind(data_type)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ================================
    // 告警标注相关函数
    // ================================

    pub async fn create_annotation(
        &self,
        alert_data_id: &str,
        annotation_type: &str,
        labels: Option<Vec<String>>,
        confidence: Option<f64>,
        is_malicious: Option<bool>,
        threat_level: Option<&str>,
        mitre_techniques: Option<Vec<String>>,
        attack_stage: Option<&str>,
        title: Option<&str>,
        description: Option<&str>,
        notes: Option<&str>,
        annotated_by: uuid::Uuid,
    ) -> Result<uuid::Uuid> {
        let annotation_id = uuid::Uuid::new_v4();
        let labels_json = labels.map(|l| serde_json::to_value(l).unwrap_or(serde_json::Value::Null));
        let mitre_techniques_json = mitre_techniques.map(|mt| serde_json::to_value(mt).unwrap_or(serde_json::Value::Null));

        sqlx::query(
            r#"
            INSERT INTO alert_annotations 
            (id, alert_data_id, annotation_type, labels, confidence, is_malicious, 
             threat_level, mitre_techniques, attack_stage, title, description, notes, annotated_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#
        )
        .bind(annotation_id)
        .bind(alert_data_id)
        .bind(annotation_type)
        .bind(labels_json)
        .bind(confidence.map(|c| c as f32))
        .bind(is_malicious)
        .bind(threat_level)
        .bind(mitre_techniques_json)
        .bind(attack_stage)
        .bind(title)
        .bind(description)
        .bind(notes)
        .bind(annotated_by)
        .execute(&self.pool)
        .await?;

        Ok(annotation_id)
    }

    pub async fn get_annotations(
        &self,
        limit: u32,
        offset: u32,
        alert_data_id: Option<&str>,
        annotation_type: Option<&str>,
        is_malicious: Option<bool>,
        threat_level: Option<&str>,
        annotated_by: Option<uuid::Uuid>,
        review_status: Option<&str>,
    ) -> Result<Vec<serde_json::Value>> {
        let mut query = r#"
            SELECT aa.*, u1.username as annotated_by_username, u2.username as reviewed_by_username
            FROM alert_annotations aa 
            LEFT JOIN users u1 ON aa.annotated_by = u1.id
            LEFT JOIN users u2 ON aa.reviewed_by = u2.id
            WHERE 1=1
        "#.to_string();

        let mut bind_count = 0;
        let mut conditions = Vec::new();

        if alert_data_id.is_some() {
            bind_count += 1;
            conditions.push(format!("aa.alert_data_id = ${}", bind_count));
        }
        if annotation_type.is_some() {
            bind_count += 1;
            conditions.push(format!("aa.annotation_type = ${}", bind_count));
        }
        if is_malicious.is_some() {
            bind_count += 1;
            conditions.push(format!("aa.is_malicious = ${}", bind_count));
        }
        if threat_level.is_some() {
            bind_count += 1;
            conditions.push(format!("aa.threat_level = ${}", bind_count));
        }
        if annotated_by.is_some() {
            bind_count += 1;
            conditions.push(format!("aa.annotated_by = ${}", bind_count));
        }
        if review_status.is_some() {
            bind_count += 1;
            conditions.push(format!("aa.review_status = ${}", bind_count));
        }

        if !conditions.is_empty() {
            query = format!("{} AND {}", query, conditions.join(" AND "));
        }

        bind_count += 1;
        let limit_param = bind_count;
        bind_count += 1;
        let offset_param = bind_count;

        query = format!("{} ORDER BY aa.annotated_at DESC LIMIT ${} OFFSET ${}", query, limit_param, offset_param);

        let mut sqlx_query = sqlx::query(&query);

        if let Some(alert_id) = alert_data_id {
            sqlx_query = sqlx_query.bind(alert_id);
        }
        if let Some(ann_type) = annotation_type {
            sqlx_query = sqlx_query.bind(ann_type);
        }
        if let Some(malicious) = is_malicious {
            sqlx_query = sqlx_query.bind(malicious);
        }
        if let Some(threat_lvl) = threat_level {
            sqlx_query = sqlx_query.bind(threat_lvl);
        }
        if let Some(annotator) = annotated_by {
            sqlx_query = sqlx_query.bind(annotator);
        }
        if let Some(review_st) = review_status {
            sqlx_query = sqlx_query.bind(review_st);
        }

        sqlx_query = sqlx_query.bind(limit as i64).bind(offset as i64);

        let rows = sqlx_query.fetch_all(&self.pool).await?;

        let mut annotations = Vec::new();
        for row in rows {
            let annotation = serde_json::json!({
                "id": row.get::<uuid::Uuid, _>("id"),
                "alert_data_id": row.get::<String, _>("alert_data_id"),
                "annotation_type": row.get::<String, _>("annotation_type"),
                "labels": row.get::<Option<serde_json::Value>, _>("labels"),
                "confidence": row.get::<Option<f32>, _>("confidence"),
                "is_malicious": row.get::<Option<bool>, _>("is_malicious"),
                "threat_level": row.get::<Option<String>, _>("threat_level"),
                "mitre_techniques": row.get::<Option<serde_json::Value>, _>("mitre_techniques"),
                "attack_stage": row.get::<Option<String>, _>("attack_stage"),
                "title": row.get::<Option<String>, _>("title"),
                "description": row.get::<Option<String>, _>("description"),
                "notes": row.get::<Option<String>, _>("notes"),
                "annotated_by": row.get::<uuid::Uuid, _>("annotated_by"),
                "annotated_by_username": row.get::<Option<String>, _>("annotated_by_username"),
                "annotated_at": row.get::<chrono::DateTime<chrono::Utc>, _>("annotated_at"),
                "reviewed_by": row.get::<Option<uuid::Uuid>, _>("reviewed_by"),
                "reviewed_by_username": row.get::<Option<String>, _>("reviewed_by_username"),
                "reviewed_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("reviewed_at"),
                "review_status": row.get::<String, _>("review_status"),
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
                "updated_at": row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at")
            });
            annotations.push(annotation);
        }

        Ok(annotations)
    }
}