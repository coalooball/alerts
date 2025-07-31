use anyhow::Result;
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use log::{info, error};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};

use alerts::{AlertMessage, KafkaConfig, KafkaProducer, Database, DatabaseConfig};
use alerts::kafka::config::{KafkaProducerConfig, KafkaConsumerConfig};

#[derive(Clone)]
struct AppState {
    config: KafkaConfig,
    database: Arc<Database>,
}

#[derive(Deserialize)]
struct TestConnectivityQuery {
    bootstrap_servers: Option<String>,
    topic: Option<String>,
}

#[derive(Serialize)]
struct ConnectivityResponse {
    success: bool,
    message: String,
    details: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct SaveConfigRequest {
    name: String,
    bootstrap_servers: String,
    topic: String,
    group_id: String,
    message_timeout_ms: Option<i32>,
    request_timeout_ms: Option<i32>,
    retry_backoff_ms: Option<i32>,
    retries: Option<i32>,
    auto_offset_reset: Option<String>,
    enable_auto_commit: Option<bool>,
    auto_commit_interval_ms: Option<i32>,
}

#[derive(Serialize)]
struct SaveConfigResponse {
    success: bool,
    message: String,
    config_id: Option<String>,
}

#[derive(Serialize)]
struct ConfigListResponse {
    success: bool,
    configs: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct ConsumeMessagesQuery {
    topic: Option<String>,
    group_id: Option<String>,
    max_messages: Option<u32>,
    timeout_ms: Option<u64>,
}

#[derive(Serialize)]
struct ConsumeMessagesResponse {
    success: bool,
    messages: Vec<KafkaMessage>,
    count: usize,
}

#[derive(Serialize)]
struct KafkaMessage {
    partition: i32,
    offset: i64,
    timestamp: Option<i64>,
    key: Option<String>,
    payload: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("🚀 Starting Kafka Alerts Web Server");

    // Initialize database connection
    let db_config = DatabaseConfig::from_env();
    info!("📊 Connecting to database at {}:{}...", db_config.host, db_config.port);
    
    let database = match Database::new(db_config).await {
        Ok(db) => {
            info!("✅ Database connection established");
            
            // Initialize database schema
            if let Err(e) = db.initialize_schema().await {
                error!("❌ Failed to initialize database schema: {}", e);
                return Err(e);
            }
            
            Arc::new(db)
        }
        Err(e) => {
            error!("❌ Failed to connect to database: {}", e);
            error!("Please ensure PostgreSQL is running with database 'alert_server'");
            return Err(e);
        }
    };

    // Test database connection
    match database.test_connection().await {
        Ok(true) => info!("✅ Database connectivity test passed"),
        Ok(false) => {
            error!("❌ Database connectivity test failed");
            return Err(anyhow::anyhow!("Database connectivity test failed"));
        }
        Err(e) => {
            error!("❌ Database connectivity test error: {}", e);
            return Err(e);
        }
    }

    // Load Kafka configuration from database
    let config = match KafkaConfig::from_database().await {
        Ok(config) => {
            info!("✅ Loaded Kafka configuration from database");
            info!("📡 Kafka broker: {}", config.bootstrap_servers);
            info!("📂 Topic: {}", config.topic);
            config
        }
        Err(e) => {
            info!("Failed to load config from database: {}. Using default configuration.", e);
            KafkaConfig::default()
        }
    };

    let state = AppState { config, database };

    let app = Router::new()
        .route("/api/test-connectivity", get(test_connectivity))
        .route("/api/config", get(get_config))
        .route("/api/configs", get(list_configs))
        .route("/api/consume-messages", get(consume_messages))
        .route("/api/config", post(save_config))
        .route("/api/config/:id/activate", post(activate_config))
        .nest_service("/", ServeDir::new("frontend/dist"))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
        )
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("🌐 Web server running on http://localhost:3000");
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn test_connectivity(
    Query(params): Query<TestConnectivityQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<ConnectivityResponse>, StatusCode> {
    info!("Testing Kafka connectivity");

    let bootstrap_servers = params.bootstrap_servers
        .unwrap_or_else(|| state.config.bootstrap_servers.clone());
    let topic = params.topic
        .unwrap_or_else(|| state.config.topic.clone());

    let mut test_config = state.config.clone();
    test_config.bootstrap_servers = bootstrap_servers.clone();
    test_config.topic = topic.clone();

    match test_kafka_connection(&test_config).await {
        Ok(details) => {
            info!("✅ Kafka connectivity test successful");
            Ok(ResponseJson(ConnectivityResponse {
                success: true,
                message: format!("Successfully connected to Kafka at {}", bootstrap_servers),
                details: Some(details),
            }))
        }
        Err(e) => {
            error!("❌ Kafka connectivity test failed: {}", e);
            Ok(ResponseJson(ConnectivityResponse {
                success: false,
                message: format!("Failed to connect to Kafka: {}", e),
                details: None,
            }))
        }
    }
}

async fn test_kafka_connection(config: &KafkaConfig) -> Result<serde_json::Value> {
    let producer = KafkaProducer::new(config.clone()).await?;
    
    // Test sending a simple message
    let test_alert = AlertMessage::info(
        format!("connectivity-test-{}", chrono::Utc::now().timestamp()),
        "Kafka connectivity test message".to_string(),
    );
    
    producer.send_alert(test_alert).await?;

    let mut details = serde_json::Map::new();
    details.insert("bootstrap_servers".to_string(), serde_json::Value::String(config.bootstrap_servers.clone()));
    details.insert("topic".to_string(), serde_json::Value::String(config.topic.clone()));
    details.insert("test_time".to_string(), serde_json::Value::String(chrono::Utc::now().to_rfc3339()));

    Ok(serde_json::Value::Object(details))
}

async fn save_config(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SaveConfigRequest>,
) -> Result<ResponseJson<SaveConfigResponse>, StatusCode> {
    info!("Saving Kafka configuration: {}", request.name);

    let kafka_config = KafkaConfig {
        bootstrap_servers: request.bootstrap_servers,
        topic: request.topic,
        group_id: request.group_id,
        producer: KafkaProducerConfig {
            message_timeout_ms: request.message_timeout_ms.unwrap_or(5000) as u32,
            request_timeout_ms: request.request_timeout_ms.unwrap_or(5000) as u32,
            retry_backoff_ms: request.retry_backoff_ms.unwrap_or(100) as u32,
            retries: request.retries.unwrap_or(3) as u32,
        },
        consumer: KafkaConsumerConfig {
            auto_offset_reset: request.auto_offset_reset.unwrap_or_else(|| "earliest".to_string()),
            enable_auto_commit: request.enable_auto_commit.unwrap_or(true),
            auto_commit_interval_ms: request.auto_commit_interval_ms.unwrap_or(1000) as u32,
        },
    };

    let config_row = kafka_config.to_database_row(request.name, false);
    
    match state.database.create_kafka_config(&config_row).await {
        Ok(id) => {
            info!("✅ Configuration saved successfully: {}", id);
            Ok(ResponseJson(SaveConfigResponse {
                success: true,
                message: "Configuration saved successfully".to_string(),
                config_id: Some(id.to_string()),
            }))
        }
        Err(e) => {
            error!("❌ Failed to save configuration: {}", e);
            Ok(ResponseJson(SaveConfigResponse {
                success: false,
                message: format!("Failed to save configuration: {}", e),
                config_id: None,
            }))
        }
    }
}

async fn list_configs(
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<ConfigListResponse>, StatusCode> {
    info!("Listing Kafka configurations");

    match state.database.list_kafka_configs().await {
        Ok(configs) => {
            let configs_json: Vec<serde_json::Value> = configs
                .into_iter()
                .map(|config| serde_json::to_value(config).unwrap_or(serde_json::Value::Null))
                .collect();
                
            Ok(ResponseJson(ConfigListResponse {
                success: true,
                configs: configs_json,
            }))
        }
        Err(e) => {
            error!("❌ Failed to list configurations: {}", e);
            Ok(ResponseJson(ConfigListResponse {
                success: false,
                configs: vec![],
            }))
        }
    }
}

async fn activate_config(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("Activating Kafka configuration: {}", id);

    let config_id = match uuid::Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(e) => {
            error!("Invalid UUID format: {}", e);
            return Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Invalid configuration ID format"
            })));
        }
    };

    match state.database.set_active_kafka_config(config_id).await {
        Ok(_) => {
            info!("✅ Configuration activated successfully: {}", id);
            Ok(ResponseJson(serde_json::json!({
                "success": true,
                "message": "Configuration activated successfully"
            })))
        }
        Err(e) => {
            error!("❌ Failed to activate configuration: {}", e);
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": format!("Failed to activate configuration: {}", e)
            })))
        }
    }
}

async fn consume_messages(
    Query(params): Query<ConsumeMessagesQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<ConsumeMessagesResponse>, StatusCode> {
    info!("Consuming messages from Kafka");

    let mut config = state.config.clone();
    if let Some(topic) = params.topic {
        config.topic = topic;
    }
    if let Some(group_id) = params.group_id {
        config.group_id = group_id;
    }

    let max_messages = params.max_messages.unwrap_or(10);
    let timeout_ms = params.timeout_ms.unwrap_or(5000);

    match consume_kafka_messages(&config, max_messages, timeout_ms).await {
        Ok(messages) => {
            info!("✅ Consumed {} messages", messages.len());
            Ok(ResponseJson(ConsumeMessagesResponse {
                success: true,
                count: messages.len(),
                messages,
            }))
        }
        Err(e) => {
            error!("❌ Failed to consume messages: {}", e);
            Ok(ResponseJson(ConsumeMessagesResponse {
                success: false,
                count: 0,
                messages: vec![],
            }))
        }
    }
}

async fn consume_kafka_messages(
    config: &KafkaConfig,
    max_messages: u32,
    timeout_ms: u64,
) -> Result<Vec<KafkaMessage>> {
    use rdkafka::{
        config::ClientConfig,
        consumer::{Consumer, StreamConsumer},
        Message,
    };
    use std::time::Duration;

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &config.bootstrap_servers)
        .set("group.id", &format!("{}-web", config.group_id))
        .set("auto.offset.reset", "latest")
        .set("enable.auto.commit", "true")
        .create()?;

    consumer.subscribe(&[&config.topic])?;

    let mut messages = Vec::new();
    let timeout = Duration::from_millis(timeout_ms);
    let start_time = std::time::Instant::now();

    while messages.len() < max_messages as usize && start_time.elapsed() < timeout {
        match tokio::time::timeout(Duration::from_millis(1000), consumer.recv()).await {
            Ok(Ok(msg)) => {
                let kafka_msg = KafkaMessage {
                    partition: msg.partition(),
                    offset: msg.offset(),
                    timestamp: msg.timestamp().to_millis(),
                    key: msg.key_view::<str>().and_then(|k| k.ok()).map(|s| s.to_string()),
                    payload: msg.payload_view::<str>()
                        .and_then(|p| p.ok())
                        .unwrap_or("<invalid payload>")
                        .to_string(),
                };
                messages.push(kafka_msg);
            }
            Ok(Err(e)) => {
                if !e.to_string().contains("timed out") {
                    error!("Consumer error: {}", e);
                    break;
                }
            }
            Err(_) => {
                // Timeout waiting for message, continue
            }
        }
    }

    Ok(messages)
}

async fn get_config(
    State(state): State<Arc<AppState>>,
) -> ResponseJson<serde_json::Value> {
    let mut config_map = serde_json::Map::new();
    config_map.insert("bootstrap_servers".to_string(), serde_json::Value::String(state.config.bootstrap_servers.clone()));
    config_map.insert("topic".to_string(), serde_json::Value::String(state.config.topic.clone()));
    config_map.insert("group_id".to_string(), serde_json::Value::String(state.config.group_id.clone()));

    ResponseJson(serde_json::Value::Object(config_map))
}