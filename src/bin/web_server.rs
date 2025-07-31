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

use alerts::{AlertMessage, EdrAlert, NgavAlert, KafkaConfig, KafkaProducer};

#[derive(Clone)]
struct AppState {
    config: KafkaConfig,
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
struct SendMessageRequest {
    message_type: String, // "alert", "edr", "ngav"
    data: serde_json::Value,
    topic: Option<String>,
}

#[derive(Serialize)]
struct SendMessageResponse {
    success: bool,
    message: String,
    message_id: Option<String>,
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

    // Load configuration
    let config = match KafkaConfig::from_file("config.toml") {
        Ok(config) => {
            info!("✅ Loaded configuration from config.toml");
            info!("📡 Kafka broker: {}", config.bootstrap_servers);
            info!("📂 Topic: {}", config.topic);
            config
        }
        Err(e) => {
            info!("Failed to load config: {}. Using default configuration.", e);
            KafkaConfig::default()
        }
    };

    let state = AppState { config };

    let app = Router::new()
        .route("/api/test-connectivity", get(test_connectivity))
        .route("/api/send-message", post(send_message))
        .route("/api/consume-messages", get(consume_messages))
        .route("/api/config", get(get_config))
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

async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SendMessageRequest>,
) -> Result<ResponseJson<SendMessageResponse>, StatusCode> {
    info!("Sending {} message to Kafka", request.message_type);

    let mut config = state.config.clone();
    if let Some(topic) = request.topic {
        config.topic = topic;
    }

    let producer = match KafkaProducer::new(config).await {
        Ok(producer) => producer,
        Err(e) => {
            error!("Failed to create Kafka producer: {}", e);
            return Ok(ResponseJson(SendMessageResponse {
                success: false,
                message: format!("Failed to create producer: {}", e),
                message_id: None,
            }));
        }
    };

    let message_id = format!("{}-{}", request.message_type, chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));

    let result = match request.message_type.as_str() {
        "alert" => {
            match serde_json::from_value::<AlertMessage>(request.data.clone()) {
                Ok(alert) => producer.send_alert(alert).await,
                Err(_) => {
                    let alert = AlertMessage::info(message_id.clone(), request.data.to_string());
                    producer.send_alert(alert).await
                }
            }
        }
        "edr" => {
            match serde_json::from_value::<EdrAlert>(request.data) {
                Ok(edr_alert) => producer.send_edr_alert(edr_alert).await,
                Err(e) => return Ok(ResponseJson(SendMessageResponse {
                    success: false,
                    message: format!("Invalid EDR alert format: {}", e),
                    message_id: None,
                })),
            }
        }
        "ngav" => {
            match serde_json::from_value::<NgavAlert>(request.data) {
                Ok(ngav_alert) => producer.send_ngav_alert(ngav_alert).await,
                Err(e) => return Ok(ResponseJson(SendMessageResponse {
                    success: false,
                    message: format!("Invalid NGAV alert format: {}", e),
                    message_id: None,
                })),
            }
        }
        _ => {
            return Ok(ResponseJson(SendMessageResponse {
                success: false,
                message: "Invalid message type. Use 'alert', 'edr', or 'ngav'".to_string(),
                message_id: None,
            }));
        }
    };

    match result {
        Ok(_) => {
            info!("✅ Message sent successfully: {}", message_id);
            Ok(ResponseJson(SendMessageResponse {
                success: true,
                message: "Message sent successfully".to_string(),
                message_id: Some(message_id),
            }))
        }
        Err(e) => {
            error!("❌ Failed to send message: {}", e);
            Ok(ResponseJson(SendMessageResponse {
                success: false,
                message: format!("Failed to send message: {}", e),
                message_id: None,
            }))
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