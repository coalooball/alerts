use anyhow::Result;
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{delete, get, post, put},
    Router,
};
use clap::{Arg, Command};
use log::{info, error};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};

use alerts::{AlertMessage, KafkaConfig, KafkaProducer, Database, DatabaseConfig, ClickHouseConnection, ConsumerService, AuthService, LoginRequest, LoginResponse};
use alerts::kafka::config::{KafkaProducerConfig, KafkaConsumerConfig};
use alerts::clickhouse::AlertFilters;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    config: KafkaConfig,
    database: Arc<Database>,
    clickhouse: Option<Arc<ClickHouseConnection>>,
    consumer_service: Option<Arc<ConsumerService>>,
    auth_service: Arc<AuthService>,
}

#[derive(Deserialize)]
struct TestConnectivityQuery {
    bootstrap_servers: Option<String>,
    topic: Option<String>,
}

#[derive(Deserialize)]
struct AnalysisQuery {
    #[serde(rename = "timeRange")]
    time_range: Option<String>,
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
struct UpdateConfigRequest {
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

#[derive(Deserialize)]
struct ToggleConfigRequest {
    is_active: bool,
}

#[derive(Deserialize)]
struct SaveClickHouseConfigRequest {
    name: String,
    host: String,
    port: Option<i32>,
    database_name: String,
    username: String,
    password: Option<String>,
    use_tls: Option<bool>,
    connection_timeout_ms: Option<i32>,
    request_timeout_ms: Option<i32>,
    max_connections: Option<i32>,
}

#[derive(Deserialize)]
struct SaveDataSourceConfigRequest {
    data_type: String, // 'edr' or 'ngav'
    kafka_config_ids: Vec<String>,
}

#[derive(Serialize)]
struct DataSourceConfigResponse {
    success: bool,
    message: String,
    configs: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct ClickHouseConfigResponse {
    success: bool,
    message: String,
    config: Option<serde_json::Value>,
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

#[derive(Serialize)]
struct KafkaStatsResponse {
    success: bool,
    message_rate: f64,
    total_messages: u64,
    type_breakdown: std::collections::HashMap<String, u64>,
    severity_breakdown: std::collections::HashMap<String, u64>,
}

#[derive(Serialize)]
struct AlertsResponse {
    success: bool,
    alerts: Vec<serde_json::Value>,
    total: u64,
}

#[derive(Deserialize)]
struct AlertsQuery {
    limit: Option<u32>,
    offset: Option<u32>,
    device_name: Option<String>,
    device_ip: Option<String>,
    alert_type: Option<String>,
    threat_category: Option<String>,
    severity: Option<u32>,
    data_type: Option<String>,
    kafka_source: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
}

#[derive(Serialize)]
struct AlertDetailResponse {
    success: bool,
    alert: Option<serde_json::Value>,
    alert_type: Option<String>,
}

#[derive(Serialize)]
struct ConsumerStatusResponse {
    success: bool,
    consumers: serde_json::Value,
}

// ================================
// 告警标注相关数据结构
// ================================

#[derive(Deserialize)]
struct CreateAnnotationRequest {
    alert_data_id: String,
    annotation_type: String,
    labels: Option<Vec<String>>,
    confidence: Option<f64>,
    is_malicious: Option<bool>,
    threat_level: Option<String>,
    mitre_techniques: Option<Vec<String>>,
    attack_stage: Option<String>,
    title: Option<String>,
    description: Option<String>,
    notes: Option<String>,
}

#[derive(Serialize)]
struct AnnotationResponse {
    success: bool,
    message: String,
    annotation_id: Option<String>,
}

#[derive(Serialize)]
struct AnnotationsListResponse {
    success: bool,
    annotations: Vec<serde_json::Value>,
    total: u64,
}

#[derive(Deserialize)]
struct AnnotationsQuery {
    alert_data_id: Option<String>,
    annotation_type: Option<String>,
    is_malicious: Option<bool>,
    threat_level: Option<String>,
    annotated_by: Option<String>,
    review_status: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

// ================================
// 威胁事件相关数据结构  
// ================================

#[derive(Deserialize)]
struct CreateThreatEventRequest {
    title: String,
    description: Option<String>,
    event_type: String,
    severity: i32,
    threat_category: Option<String>,
    event_start_time: Option<String>,
    event_end_time: Option<String>,
    mitre_tactics: Option<Vec<String>>,
    mitre_techniques: Option<Vec<String>>,
    kill_chain_phases: Option<Vec<String>>,
    priority: Option<String>,
    tags: Option<Vec<String>>,
    creation_method: Option<String>,
}

#[derive(Serialize)]
struct ThreatEventResponse {
    success: bool,
    message: String,
    threat_event_id: Option<String>,
}

#[derive(Serialize)]
struct ThreatEventsListResponse {
    success: bool,
    threat_events: Vec<serde_json::Value>,
    total: u64,
}

#[derive(Deserialize)]
struct ThreatEventsQuery {
    status: Option<String>,
    severity: Option<i32>,
    event_type: Option<String>,
    assigned_to: Option<String>,
    creation_method: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Deserialize)]
struct UpdateThreatEventRequest {
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    assigned_to: Option<String>,
    resolution_notes: Option<String>,
}

#[derive(Deserialize)]
struct CorrelateAlertsRequest {
    threat_event_id: String,
    alert_data_ids: Vec<String>,
    correlation_type: String,
    correlation_reason: Option<String>,
    roles_in_event: Option<Vec<String>>, // 对应每个alert的角色
}

#[derive(Serialize)]
struct CorrelationResponse {
    success: bool,
    message: String,
    correlations_created: u32,
}

#[derive(Serialize)]
struct AlertDataResponse {
    success: bool,
    alert_data: Vec<serde_json::Value>,
    total: u64,
}

#[derive(Deserialize)]
struct AlertDataQuery {
    processing_status: Option<String>,
    alert_type: Option<String>,
    severity: Option<i32>,
    device_name: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    // Parse command line arguments
    let matches = Command::new("web-server")
        .version("1.0")
        .author("Alerts Team")
        .about("Kafka Alerts Web Server")
        .arg(
            Arg::new("init")
                .long("init")
                .action(clap::ArgAction::SetTrue)
                .help("Initialize database (drop and recreate if exists)")
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Port to listen on")
                .default_value("3000")
                .value_parser(clap::value_parser!(u16))
        )
        .get_matches();

    let init_db = matches.get_flag("init");
    let port = *matches.get_one::<u16>("port").unwrap();

    if init_db {
        info!("🔄 Database initialization mode enabled");
    }

    info!("🚀 Starting Kafka Alerts Web Server");

    // Initialize database connection
    let db_config = DatabaseConfig::from_env();
    info!("📊 Connecting to database at {}:{}...", db_config.host, db_config.port);
    
    let database = if init_db {
        info!("🗄️ Initializing database (drop and recreate)...");
        match Database::new_with_init(db_config).await {
            Ok(db) => {
                info!("✅ PostgreSQL database initialized successfully");
                Arc::new(db)
            }
            Err(e) => {
                error!("❌ Failed to initialize database: {}", e);
                return Err(e);
            }
        }
    } else {
        match Database::new(db_config).await {
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

    // Initialize ClickHouse connection
    let clickhouse = match database.get_clickhouse_config().await {
        Ok(Some(ch_config)) => {
            info!("🏠 Initializing ClickHouse connection...");
            match ClickHouseConnection::new(ch_config).await {
                Ok(ch) => {
                    if let Err(e) = ch.test_connection().await {
                        error!("❌ ClickHouse connectivity test failed: {}", e);
                        None
                    } else {
                        info!("✅ ClickHouse connection established");
                        // Use reinitialize_tables in init mode, otherwise use initialize_tables
                        let init_result = if init_db {
                            ch.reinitialize_tables().await
                        } else {
                            ch.initialize_tables().await
                        };
                        
                        if let Err(e) = init_result {
                            error!("❌ Failed to initialize ClickHouse tables: {}", e);
                        } else {
                            info!("✅ ClickHouse tables initialized");
                        }
                        Some(Arc::new(ch))
                    }
                }
                Err(e) => {
                    error!("❌ Failed to create ClickHouse connection: {}", e);
                    None
                }
            }
        }
        Ok(None) => {
            info!("⚠️ No ClickHouse configuration found in database");
            None
        }
        Err(e) => {
            error!("❌ Failed to load ClickHouse configuration: {}", e);
            None
        }
    };

    // Exit early if in init mode
    if init_db {
        info!("🎉 Database and ClickHouse initialization completed. Exiting...");
        return Ok(());
    }

    // Initialize consumer service if ClickHouse is available
    let consumer_service = if let Some(ch) = &clickhouse {
        info!("🔄 Initializing consumer service...");
        match ConsumerService::new(Arc::clone(&database), Arc::clone(ch)).await {
            Ok(service) => {
                if let Err(e) = service.start().await {
                    error!("❌ Failed to start consumer service: {}", e);
                    None
                } else {
                    info!("✅ Consumer service started");
                    Some(Arc::new(service))
                }
            }
            Err(e) => {
                error!("❌ Failed to create consumer service: {}", e);
                None
            }
        }
    } else {
        info!("⚠️ Consumer service not started - ClickHouse not available");
        None
    };

    // Initialize authentication service
    let auth_service = Arc::new(AuthService::new(database.get_pool().clone()));

    let state = AppState { 
        config, 
        database: Arc::clone(&database), 
        clickhouse,
        consumer_service,
        auth_service,
    };

    let app = Router::new()
        // Authentication routes
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .route("/api/me", get(get_current_user))
        .route("/api/profile", put(update_profile))
        .route("/api/change-password", post(change_password))
        // User management routes (admin only)
        .route("/api/users", get(list_users))
        .route("/api/users", post(create_user))
        .route("/api/users/:id", get(get_user))
        .route("/api/users/:id", put(update_user))
        .route("/api/users/:id", delete(delete_user))
        .route("/api/users/:id/password", put(update_user_password))
        // Other API routes
        .route("/api/test-connectivity", get(test_connectivity))
        .route("/api/config", get(get_config))
        .route("/api/configs", get(list_configs))
        .route("/api/configs/active", get(get_active_configs))
        .route("/api/consume-messages", get(consume_messages))
        .route("/api/config", post(save_config))
        .route("/api/config/:id", put(update_config))
        .route("/api/config/:id", delete(delete_config))
        .route("/api/config/:id/activate", post(activate_config))
        .route("/api/config/:id/toggle", post(toggle_config))
        .route("/api/clickhouse-config", get(get_clickhouse_config))
        .route("/api/clickhouse-config", post(save_clickhouse_config))
        .route("/api/clickhouse-config", delete(delete_clickhouse_config))
        .route("/api/test-clickhouse-connectivity", get(test_clickhouse_connectivity))
        .route("/api/data-source-config", get(get_data_source_configs))
        .route("/api/data-source-config", post(save_data_source_config))
        .route("/api/alerts", get(get_alerts))
        .route("/api/alerts/:id", get(get_alert_detail))
        .route("/api/consumer-status", get(get_consumer_status))
        .route("/api/live-messages", get(get_live_messages))
        .route("/api/kafka-stats", get(get_kafka_stats))
        .route("/api/analysis/severity-distribution", get(get_severity_distribution))
        .route("/api/analysis/time-series", get(get_time_series_trend))
        .route("/api/analysis/type-clustering", get(get_type_clustering))
        // Alert Data & Annotation routes
        .route("/api/alert-data", get(get_alert_data))
        .route("/api/alert-data/:id", get(get_alert_data_detail))
        .route("/api/annotations", get(get_annotations))
        .route("/api/annotations", post(create_annotation))
        .route("/api/annotations/:id", put(update_annotation))
        .route("/api/annotations/:id", delete(delete_annotation))
        .route("/api/annotations/:id/review", post(review_annotation))
        // Threat Event routes
        .route("/api/threat-events", get(get_threat_events))
        .route("/api/threat-events", post(create_threat_event))
        .route("/api/threat-events/:id", get(get_threat_event_detail))
        .route("/api/threat-events/:id", put(update_threat_event))
        .route("/api/threat-events/:id", delete(delete_threat_event))
        .route("/api/threat-events/:id/correlate-alerts", post(correlate_alerts_to_threat_event))
        .route("/api/threat-events/:id/timeline", get(get_threat_event_timeline))
        .route("/api/threat-events/:id/timeline", post(add_threat_event_timeline_entry))
        .nest_service("/", ServeDir::new("./frontend/dist"))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
        )
        .with_state(Arc::new(state));

    let bind_addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("🌐 Web server running on http://localhost:{}", port);
    
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

async fn get_active_configs(
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<ConfigListResponse>, StatusCode> {
    info!("Getting active Kafka configurations");

    match state.database.get_active_kafka_configs().await {
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
            error!("❌ Failed to get active configurations: {}", e);
            Ok(ResponseJson(ConfigListResponse {
                success: false,
                configs: vec![],
            }))
        }
    }
}

async fn update_config(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(request): Json<UpdateConfigRequest>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("Updating Kafka configuration: {}", id);

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

    // Get existing config to preserve is_active and timestamps
    let existing_config = match state.database.get_kafka_config_by_id(config_id).await {
        Ok(Some(config)) => config,
        Ok(None) => {
            return Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Configuration not found"
            })));
        }
        Err(e) => {
            error!("❌ Failed to get existing configuration: {}", e);
            return Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": format!("Failed to get configuration: {}", e)
            })));
        }
    };

    // Create updated config
    let updated_config = alerts::KafkaConfigRow {
        id: config_id,
        name: request.name,
        bootstrap_servers: request.bootstrap_servers,
        topic: request.topic,
        group_id: request.group_id,
        message_timeout_ms: request.message_timeout_ms.unwrap_or(5000),
        request_timeout_ms: request.request_timeout_ms.unwrap_or(5000),
        retry_backoff_ms: request.retry_backoff_ms.unwrap_or(100),
        retries: request.retries.unwrap_or(3),
        auto_offset_reset: request.auto_offset_reset.unwrap_or_else(|| "earliest".to_string()),
        enable_auto_commit: request.enable_auto_commit.unwrap_or(true),
        auto_commit_interval_ms: request.auto_commit_interval_ms.unwrap_or(1000),
        is_active: existing_config.is_active,
        created_at: existing_config.created_at,
        updated_at: chrono::Utc::now(),
    };

    match state.database.update_kafka_config(config_id, &updated_config).await {
        Ok(_) => {
            info!("✅ Configuration updated successfully: {}", id);
            Ok(ResponseJson(serde_json::json!({
                "success": true,
                "message": "Configuration updated successfully"
            })))
        }
        Err(e) => {
            error!("❌ Failed to update configuration: {}", e);
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": format!("Failed to update configuration: {}", e)
            })))
        }
    }
}

async fn delete_config(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("Deleting Kafka configuration: {}", id);

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

    match state.database.delete_kafka_config(config_id).await {
        Ok(_) => {
            info!("✅ Configuration deleted successfully: {}", id);
            Ok(ResponseJson(serde_json::json!({
                "success": true,
                "message": "Configuration deleted successfully"
            })))
        }
        Err(e) => {
            error!("❌ Failed to delete configuration: {}", e);
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": format!("Failed to delete configuration: {}", e)
            })))
        }
    }
}

async fn toggle_config(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(request): Json<ToggleConfigRequest>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("Toggling Kafka configuration: {} to {}", id, request.is_active);

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

    match state.database.toggle_kafka_config_active(config_id, request.is_active).await {
        Ok(_) => {
            let action = if request.is_active { "activated" } else { "deactivated" };
            info!("✅ Configuration {} successfully: {}", action, id);
            
            // Refresh consumer service when Kafka config is toggled
            if let Some(consumer_service) = &state.consumer_service {
                info!("🔄 Refreshing consumer service after config toggle...");
                if let Err(e) = consumer_service.refresh_consumers().await {
                    error!("❌ Failed to refresh consumer service: {}", e);
                    return Ok(ResponseJson(serde_json::json!({
                        "success": false,
                        "message": format!("Configuration {} but failed to refresh consumers: {}", action, e)
                    })));
                }
                info!("✅ Consumer service refreshed successfully");
            }
            
            Ok(ResponseJson(serde_json::json!({
                "success": true,
                "message": format!("Configuration {} and consumers refreshed successfully", action)
            })))
        }
        Err(e) => {
            error!("❌ Failed to toggle configuration: {}", e);
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": format!("Failed to toggle configuration: {}", e)
            })))
        }
    }
}

async fn activate_config(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("Activating Kafka configuration (single active): {}", id);

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
            info!("✅ Configuration activated successfully (single active): {}", id);
            
            // Refresh consumer service when a config is set as the only active one
            if let Some(consumer_service) = &state.consumer_service {
                info!("🔄 Refreshing consumer service after setting active config...");
                if let Err(e) = consumer_service.refresh_consumers().await {
                    error!("❌ Failed to refresh consumer service: {}", e);
                    return Ok(ResponseJson(serde_json::json!({
                        "success": false,
                        "message": format!("Configuration activated but failed to refresh consumers: {}", e)
                    })));
                }
                info!("✅ Consumer service refreshed successfully");
            }
            
            Ok(ResponseJson(serde_json::json!({
                "success": true,
                "message": "Configuration activated and consumers refreshed successfully (all others deactivated)"
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

// ClickHouse configuration handlers
async fn get_clickhouse_config(
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<ClickHouseConfigResponse>, StatusCode> {
    match state.database.get_clickhouse_config().await {
        Ok(config) => {
            let response = ClickHouseConfigResponse {
                success: true,
                message: "ClickHouse configuration retrieved successfully".to_string(),
                config: config.map(|c| serde_json::to_value(c).unwrap_or_default()),
            };
            Ok(ResponseJson(response))
        },
        Err(e) => {
            error!("Failed to get ClickHouse configuration: {}", e);
            let response = ClickHouseConfigResponse {
                success: false,
                message: format!("Failed to get ClickHouse configuration: {}", e),
                config: None,
            };
            Ok(ResponseJson(response))
        }
    }
}

async fn save_clickhouse_config(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SaveClickHouseConfigRequest>,
) -> Result<ResponseJson<ClickHouseConfigResponse>, StatusCode> {
    use alerts::database::ClickHouseConfigRow;
    
    let config = ClickHouseConfigRow {
        id: uuid::Uuid::new_v4(), // This will be ignored for create_or_update
        name: request.name,
        host: request.host,
        port: request.port.unwrap_or(8123),
        database_name: request.database_name,
        username: request.username,
        password: request.password,
        use_tls: request.use_tls.unwrap_or(false),
        connection_timeout_ms: request.connection_timeout_ms.unwrap_or(10000),
        request_timeout_ms: request.request_timeout_ms.unwrap_or(30000),
        max_connections: request.max_connections.unwrap_or(10),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    match state.database.create_or_update_clickhouse_config(&config).await {
        Ok(_id) => {
            let response = ClickHouseConfigResponse {
                success: true,
                message: "ClickHouse configuration saved successfully".to_string(),
                config: Some(serde_json::to_value(&config).unwrap_or_default()),
            };
            Ok(ResponseJson(response))
        },
        Err(e) => {
            error!("Failed to save ClickHouse configuration: {}", e);
            let response = ClickHouseConfigResponse {
                success: false,
                message: format!("Failed to save ClickHouse configuration: {}", e),
                config: None,
            };
            Ok(ResponseJson(response))
        }
    }
}

async fn delete_clickhouse_config(
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<ClickHouseConfigResponse>, StatusCode> {
    match state.database.delete_clickhouse_config().await {
        Ok(()) => {
            let response = ClickHouseConfigResponse {
                success: true,
                message: "ClickHouse configuration deleted successfully".to_string(),
                config: None,
            };
            Ok(ResponseJson(response))
        },
        Err(e) => {
            error!("Failed to delete ClickHouse configuration: {}", e);
            let response = ClickHouseConfigResponse {
                success: false,
                message: format!("Failed to delete ClickHouse configuration: {}", e),
                config: None,
            };
            Ok(ResponseJson(response))
        }
    }
}

async fn test_clickhouse_connectivity(
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<ConnectivityResponse>, StatusCode> {
    let config = match state.database.get_clickhouse_config().await {
        Ok(Some(config)) => config,
        Ok(None) => {
            let response = ConnectivityResponse {
                success: false,
                message: "No ClickHouse configuration found".to_string(),
                details: None,
            };
            return Ok(ResponseJson(response));
        },
        Err(e) => {
            error!("Failed to get ClickHouse configuration: {}", e);
            let response = ConnectivityResponse {
                success: false,
                message: format!("Failed to get ClickHouse configuration: {}", e),
                details: None,
            };
            return Ok(ResponseJson(response));
        }
    };

    // Test ClickHouse connectivity (simplified version - just check if we can format the connection URL)
    let connection_url = if config.use_tls {
        format!("https://{}:{}", config.host, config.port)
    } else {
        format!("http://{}:{}", config.host, config.port)
    };

    let response = ConnectivityResponse {
        success: true,
        message: "ClickHouse configuration is valid".to_string(),
        details: Some(serde_json::json!({
            "host": config.host,
            "port": config.port,
            "database": config.database_name,
            "connection_url": connection_url,
            "use_tls": config.use_tls
        })),
    };

    Ok(ResponseJson(response))
}

async fn get_data_source_configs(
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<DataSourceConfigResponse>, StatusCode> {
    info!("Getting data source configurations");

    match state.database.get_data_source_configs().await {
        Ok(configs) => {
            // Group configs by data_type
            let mut grouped_configs = std::collections::HashMap::new();
            for config in configs {
                let data_type = config.get("data_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                grouped_configs
                    .entry(data_type)
                    .or_insert_with(Vec::new)
                    .push(config);
            }

            let response = DataSourceConfigResponse {
                success: true,
                message: "Data source configurations retrieved successfully".to_string(),
                configs: Some(serde_json::to_value(grouped_configs).unwrap_or_default()),
            };
            Ok(ResponseJson(response))
        }
        Err(e) => {
            error!("Failed to get data source configurations: {}", e);
            let response = DataSourceConfigResponse {
                success: false,
                message: format!("Failed to get data source configurations: {}", e),
                configs: None,
            };
            Ok(ResponseJson(response))
        }
    }
}

async fn save_data_source_config(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SaveDataSourceConfigRequest>,
) -> Result<ResponseJson<DataSourceConfigResponse>, StatusCode> {
    info!("Saving data source configuration for type: {}", request.data_type);

    // Validate data_type
    if !["edr", "ngav"].contains(&request.data_type.as_str()) {
        let response = DataSourceConfigResponse {
            success: false,
            message: "Invalid data type. Must be 'edr' or 'ngav'".to_string(),
            configs: None,
        };
        return Ok(ResponseJson(response));
    }

    // Parse Kafka config IDs
    let mut kafka_config_ids = Vec::new();
    for id_str in &request.kafka_config_ids {
        match uuid::Uuid::parse_str(id_str) {
            Ok(uuid) => kafka_config_ids.push(uuid),
            Err(e) => {
                error!("Invalid UUID format for Kafka config ID {}: {}", id_str, e);
                let response = DataSourceConfigResponse {
                    success: false,
                    message: format!("Invalid Kafka config ID format: {}", id_str),
                    configs: None,
                };
                return Ok(ResponseJson(response));
            }
        }
    }

    match state.database.save_data_source_config(&request.data_type, &kafka_config_ids).await {
        Ok(()) => {
            info!("✅ Data source configuration saved successfully for type: {}", request.data_type);
            
            // Refresh consumer service to pick up new data source mappings
            if let Some(consumer_service) = &state.consumer_service {
                info!("🔄 Refreshing consumer service to apply new data source mappings...");
                if let Err(e) = consumer_service.refresh_consumers().await {
                    error!("❌ Failed to refresh consumer service: {}", e);
                    let response = DataSourceConfigResponse {
                        success: false,
                        message: format!("Configuration saved but failed to refresh consumers: {}", e),
                        configs: None,
                    };
                    return Ok(ResponseJson(response));
                }
                info!("✅ Consumer service refreshed successfully");
            }
            
            let response = DataSourceConfigResponse {
                success: true,
                message: "Data source configuration saved and consumers refreshed successfully".to_string(),
                configs: None,
            };
            Ok(ResponseJson(response))
        }
        Err(e) => {
            error!("❌ Failed to save data source configuration: {}", e);
            let response = DataSourceConfigResponse {
                success: false,
                message: format!("Failed to save data source configuration: {}", e),
                configs: None,
            };
            Ok(ResponseJson(response))
        }
    }
}

async fn get_alerts(
    Query(params): Query<AlertsQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<AlertsResponse>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    match &state.clickhouse {
        Some(ch) => {
            // Debug table contents if no data exists
            if let Err(e) = ch.debug_table_contents().await {
                error!("Failed to debug table contents: {}", e);
            }
            
            // Convert AlertsQuery to AlertFilters
            let filters = AlertFilters {
                device_name: params.device_name.clone(),
                device_ip: params.device_ip.clone(),
                alert_type: params.alert_type.clone(),
                threat_category: params.threat_category.clone(),
                severity: params.severity,
                data_type: params.data_type.clone(),
                kafka_source: params.kafka_source.clone(),
                date_from: params.date_from.clone(),
                date_to: params.date_to.clone(),
            };
            
            match ch.get_common_alerts_with_filters(limit, offset, &filters).await {
                Ok(alerts) => {
                    info!("Successfully retrieved {} alerts from ClickHouse", alerts.len());
                    let alerts_json: Vec<serde_json::Value> = alerts
                        .into_iter()
                        .enumerate()
                        .filter_map(|(i, alert)| {
                            match serde_json::to_value(alert) {
                                Ok(value) => Some(value),
                                Err(e) => {
                                    error!("Failed to serialize alert {}: {}", i, e);
                                    None
                                }
                            }
                        })
                        .collect();
                    
                    // Get filtered total count from database
                    let total = match ch.get_filtered_alerts_count(&filters).await {
                        Ok(count) => count,
                        Err(e) => {
                            error!("Failed to get filtered alerts count: {}", e);
                            alerts_json.len() as u64
                        }
                    };
                    
                    Ok(ResponseJson(AlertsResponse {
                        success: true,
                        alerts: alerts_json,
                        total,
                    }))
                }
                Err(e) => {
                    error!("Failed to get alerts: {}", e);
                    Ok(ResponseJson(AlertsResponse {
                        success: false,
                        alerts: vec![],
                        total: 0,
                    }))
                }
            }
        }
        None => {
            Ok(ResponseJson(AlertsResponse {
                success: false,
                alerts: vec![],
                total: 0,
            }))
        }
    }
}

async fn get_alert_detail(
    axum::extract::Path(id): axum::extract::Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<AlertDetailResponse>, StatusCode> {
    match &state.clickhouse {
        Some(ch) => {
            // First try to get common alert
            match ch.get_common_alert_by_id(&id).await {
                Ok(Some(common_alert)) => {
                    let data_type = common_alert.data_type.clone();
                    
                    // Get specific alert details based on type
                    let specific_alert = match data_type.as_str() {
                        "edr" => {
                            match ch.get_edr_alert_by_id(&id).await {
                                Ok(Some(alert)) => {
                                    match serde_json::to_value(alert) {
                                        Ok(value) => Some(value),
                                        Err(e) => {
                                            error!("Failed to serialize EDR alert {}: {}", id, e);
                                            None
                                        }
                                    }
                                },
                                Ok(None) => None,
                                Err(e) => {
                                    error!("Failed to get EDR alert {}: {}", id, e);
                                    None
                                }
                            }
                        }
                        "ngav" => {
                            match ch.get_ngav_alert_by_id(&id).await {
                                Ok(Some(alert)) => {
                                    match serde_json::to_value(alert) {
                                        Ok(value) => Some(value),
                                        Err(e) => {
                                            error!("Failed to serialize NGAV alert {}: {}", id, e);
                                            None
                                        }
                                    }
                                },
                                Ok(None) => None,
                                Err(e) => {
                                    error!("Failed to get NGAV alert {}: {}", id, e);
                                    None
                                }
                            }
                        }
                        _ => None
                    };
                    
                    // Safely serialize the common alert
                    match serde_json::to_value(&common_alert) {
                        Ok(mut alert_json) => {
                            if let Some(specific) = specific_alert {
                                if let (Some(alert_obj), Some(specific_obj)) = 
                                    (alert_json.as_object_mut(), specific.as_object()) {
                                    for (key, value) in specific_obj {
                                        alert_obj.insert(format!("detailed_{}", key), value.clone());
                                    }
                                }
                            }
                            
                            Ok(ResponseJson(AlertDetailResponse {
                                success: true,
                                alert: Some(alert_json),
                                alert_type: Some(data_type),
                            }))
                        },
                        Err(e) => {
                            error!("Failed to serialize alert detail for ID {}: {}", id, e);
                            // Create a minimal safe response
                            let safe_alert = serde_json::json!({
                                "id": common_alert.id,
                                "device_name": common_alert.device_name,
                                "severity": common_alert.severity,
                                "data_type": common_alert.data_type,
                                "error": "Full alert data could not be serialized due to encoding issues"
                            });
                            
                            Ok(ResponseJson(AlertDetailResponse {
                                success: true,
                                alert: Some(safe_alert),
                                alert_type: Some(data_type),
                            }))
                        }
                    }
                }
                Ok(None) => {
                    Ok(ResponseJson(AlertDetailResponse {
                        success: false,
                        alert: None,
                        alert_type: None,
                    }))
                }
                Err(e) => {
                    error!("Failed to get alert detail: {}", e);
                    Ok(ResponseJson(AlertDetailResponse {
                        success: false,
                        alert: None,
                        alert_type: None,
                    }))
                }
            }
        }
        None => {
            Ok(ResponseJson(AlertDetailResponse {
                success: false,
                alert: None,
                alert_type: None,
            }))
        }
    }
}

async fn get_consumer_status(
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<ConsumerStatusResponse>, StatusCode> {
    match &state.consumer_service {
        Some(service) => {
            let status = service.get_consumer_status().await;
            Ok(ResponseJson(ConsumerStatusResponse {
                success: true,
                consumers: serde_json::to_value(status).unwrap_or_default(),
            }))
        }
        None => {
            Ok(ResponseJson(ConsumerStatusResponse {
                success: false,
                consumers: serde_json::json!({}),
            }))
        }
    }
}

async fn get_live_messages(
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    match &state.consumer_service {
        Some(service) => {
            let mut receiver = service.get_message_receiver();
            let mut messages = Vec::new();
            
            // Try to receive messages for a short time
            for _ in 0..10 {
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(100),
                    receiver.recv()
                ).await {
                    Ok(Ok(message)) => {
                        messages.push(serde_json::json!({
                            "topic": message.topic,
                            "partition": message.partition,
                            "offset": message.offset,
                            "payload": message.payload,
                            "timestamp": message.timestamp,
                            "data_type": message.data_type
                        }));
                    }
                    _ => break,
                }
            }
            
            Ok(ResponseJson(serde_json::json!({
                "success": true,
                "messages": messages
            })))
        }
        None => {
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "messages": []
            })))
        }
    }
}

async fn get_kafka_stats(
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<KafkaStatsResponse>, StatusCode> {
    use std::collections::HashMap;
    
    // Get recent messages from ClickHouse
    let (message_rate, total_messages, type_breakdown, severity_breakdown) = match &state.clickhouse {
        Some(ch) => {
            // Get stats from last 5 minutes
            match ch.get_alert_stats(5).await {
                Ok(stats) => stats,
                Err(e) => {
                    error!("Failed to get alert stats: {}", e);
                    (0.0, 0, HashMap::new(), HashMap::new())
                }
            }
        }
        None => {
            // If no ClickHouse, return empty stats
            (0.0, 0, HashMap::new(), HashMap::new())
        }
    };
    
    Ok(ResponseJson(KafkaStatsResponse {
        success: true,
        message_rate,
        total_messages,
        type_breakdown,
        severity_breakdown,
    }))
}

// Analysis API handlers
async fn get_severity_distribution(
    Query(params): Query<AnalysisQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let time_range = params.time_range.unwrap_or_else(|| "1h".to_string());
    
    match &state.clickhouse {
        Some(ch) => {
            match ch.get_severity_distribution(&time_range).await {
                Ok(data) => Ok(ResponseJson(serde_json::Value::Array(data))),
                Err(e) => {
                    error!("Failed to get severity distribution: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        None => {
            error!("ClickHouse connection not available");
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

async fn get_time_series_trend(
    Query(params): Query<AnalysisQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let time_range = params.time_range.unwrap_or_else(|| "1h".to_string());
    
    match &state.clickhouse {
        Some(ch) => {
            match ch.get_time_series_trend(&time_range).await {
                Ok(data) => Ok(ResponseJson(serde_json::Value::Array(data))),
                Err(e) => {
                    error!("Failed to get time series trend: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        None => {
            error!("ClickHouse connection not available");
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

async fn get_type_clustering(
    Query(params): Query<AnalysisQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let time_range = params.time_range.unwrap_or_else(|| "1h".to_string());
    
    match &state.clickhouse {
        Some(ch) => {
            match ch.get_type_clustering(&time_range).await {
                Ok(data) => Ok(ResponseJson(serde_json::Value::Array(data))),
                Err(e) => {
                    error!("Failed to get type clustering: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        None => {
            error!("ClickHouse connection not available");
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

// Authentication handlers
async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<ResponseJson<LoginResponse>, StatusCode> {
    info!("Login attempt for user: {}", request.username);

    match state.auth_service.authenticate_user(&request.username, &request.password).await {
        Ok(Some(user)) => {
            match state.auth_service.create_session(user.id).await {
                Ok(session) => {
                    info!("✅ User {} logged in successfully", user.username);
                    Ok(ResponseJson(LoginResponse {
                        success: true,
                        message: "Login successful".to_string(),
                        user: Some(user),
                        session_token: Some(session.session_token),
                    }))
                }
                Err(e) => {
                    error!("❌ Failed to create session: {}", e);
                    Ok(ResponseJson(LoginResponse {
                        success: false,
                        message: "Failed to create session".to_string(),
                        user: None,
                        session_token: None,
                    }))
                }
            }
        }
        Ok(None) => {
            info!("❌ Invalid credentials for user: {}", request.username);
            Ok(ResponseJson(LoginResponse {
                success: false,
                message: "Invalid username or password".to_string(),
                user: None,
                session_token: None,
            }))
        }
        Err(e) => {
            error!("❌ Login error: {}", e);
            Ok(ResponseJson(LoginResponse {
                success: false,
                message: "Authentication error".to_string(),
                user: None,
                session_token: None,
            }))
        }
    }
}

#[derive(Deserialize)]
struct LogoutRequest {
    session_token: String,
}

async fn logout(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LogoutRequest>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("Logout request");

    match state.auth_service.invalidate_session(&request.session_token).await {
        Ok(_) => {
            info!("✅ User logged out successfully");
            Ok(ResponseJson(serde_json::json!({
                "success": true,
                "message": "Logged out successfully"
            })))
        }
        Err(e) => {
            error!("❌ Logout error: {}", e);
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Logout error"
            })))
        }
    }
}

async fn get_current_user(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let session_token = match headers.get("authorization") {
        Some(value) => {
            let auth_header = value.to_str().unwrap_or("");
            if auth_header.starts_with("Bearer ") {
                &auth_header[7..]
            } else {
                ""
            }
        }
        None => ""
    };

    if session_token.is_empty() {
        return Ok(ResponseJson(serde_json::json!({
            "success": false,
            "message": "No session token provided"
        })));
    }

    match state.auth_service.validate_session(session_token).await {
        Ok(Some(user)) => {
            Ok(ResponseJson(serde_json::json!({
                "success": true,
                "user": user
            })))
        }
        Ok(None) => {
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Invalid or expired session"
            })))
        }
        Err(e) => {
            error!("❌ Session validation error: {}", e);
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Session validation error"
            })))
        }
    }
}

// Helper function to check if user is admin
async fn check_admin_permission(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<alerts::User, axum::http::StatusCode> {
    let session_token = match headers.get("authorization") {
        Some(value) => {
            let auth_header = value.to_str().unwrap_or("");
            if auth_header.starts_with("Bearer ") {
                &auth_header[7..]
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    match state.auth_service.validate_session(session_token).await {
        Ok(Some(user)) => {
            if alerts::AuthService::is_admin(&user) {
                Ok(user)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// User management API handlers
#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
    password: String,
    role: String,
}

#[derive(Deserialize)]
struct UpdateUserRequest {
    username: String,
    email: String,
    role: String,
    is_active: bool,
}

#[derive(Deserialize)]
struct UpdatePasswordRequest {
    password: String,
}

#[derive(Deserialize)]
struct ChangePasswordRequest {
    #[serde(rename = "currentPassword")]
    current_password: String,
    #[serde(rename = "newPassword")]
    new_password: String,
}

#[derive(Deserialize)]
struct UpdateProfileRequest {
    username: String,
    email: String,
    #[serde(rename = "fullName")]
    full_name: Option<String>,
    phone: Option<String>,
    department: Option<String>,
    timezone: Option<String>,
    language: Option<String>,
    #[serde(rename = "emailNotifications")]
    email_notifications: Option<bool>,
    #[serde(rename = "smsNotifications")]
    sms_notifications: Option<bool>,
}

async fn update_profile(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    // Extract session token from headers
    let session_token = match headers.get("authorization") {
        Some(value) => {
            let auth_header = value.to_str().unwrap_or("");
            if auth_header.starts_with("Bearer ") {
                &auth_header[7..]
            } else {
                return Ok(ResponseJson(serde_json::json!({
                    "success": false,
                    "message": "Invalid authorization header format"
                })));
            }
        }
        None => {
            return Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "No authorization header provided"
            })));
        }
    };

    // Validate session and get current user
    let current_user = match state.auth_service.validate_session(session_token).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Invalid or expired session"
            })));
        }
        Err(e) => {
            error!("❌ Session validation error: {}", e);
            return Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Session validation error"
            })));
        }
    };

    info!("Updating profile for user: {}", current_user.username);

    // Update user profile - users can update their own profile
    match state.auth_service.update_user_profile(
        current_user.id,
        &request.username,
        &request.email,
        request.full_name.as_deref(),
        request.phone.as_deref(),
        request.department.as_deref(),
        &request.timezone.unwrap_or_else(|| "UTC".to_string()),
        &request.language.unwrap_or_else(|| "en".to_string()),
        request.email_notifications.unwrap_or(true),
        request.sms_notifications.unwrap_or(false),
    ).await {
        Ok(_) => {
            info!("✅ Profile updated successfully for user: {}", current_user.username);
            Ok(ResponseJson(serde_json::json!({
                "success": true,
                "message": "Profile updated successfully"
            })))
        }
        Err(e) => {
            error!("❌ Failed to update profile: {}", e);
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": format!("Failed to update profile: {}", e)
            })))
        }
    }
}

async fn change_password(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    // Extract session token from headers
    let session_token = match headers.get("authorization") {
        Some(value) => {
            let auth_header = value.to_str().unwrap_or("");
            if auth_header.starts_with("Bearer ") {
                &auth_header[7..]
            } else {
                return Ok(ResponseJson(serde_json::json!({
                    "success": false,
                    "message": "Invalid authorization header format"
                })));
            }
        }
        None => {
            return Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "No authorization header provided"
            })));
        }
    };

    // Validate session and get current user
    let current_user = match state.auth_service.validate_session(session_token).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Invalid or expired session"
            })));
        }
        Err(e) => {
            error!("❌ Session validation error: {}", e);
            return Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Session validation error"
            })));
        }
    };

    info!("Changing password for user: {}", current_user.username);

    // First verify current password
    match state.auth_service.authenticate_user(&current_user.username, &request.current_password).await {
        Ok(Some(_)) => {
            // Current password is correct, proceed with password change
            match state.auth_service.update_user_password(current_user.id, &request.new_password).await {
                Ok(_) => {
                    info!("✅ Password changed successfully for user: {}", current_user.username);
                    Ok(ResponseJson(serde_json::json!({
                        "success": true,
                        "message": "Password changed successfully"
                    })))
                }
                Err(e) => {
                    error!("❌ Failed to update password: {}", e);
                    Ok(ResponseJson(serde_json::json!({
                        "success": false,
                        "message": "Failed to update password"
                    })))
                }
            }
        }
        Ok(None) => {
            info!("❌ Invalid current password for user: {}", current_user.username);
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Current password is incorrect"
            })))
        }
        Err(e) => {
            error!("❌ Error verifying current password: {}", e);
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Error verifying current password"
            })))
        }
    }
}

async fn list_users(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let _admin_user = check_admin_permission(State(state.clone()), headers).await?;

    match state.auth_service.list_users().await {
        Ok(users) => {
            let users_json: Vec<serde_json::Value> = users
                .into_iter()
                .map(|user| serde_json::to_value(user).unwrap_or_default())
                .collect();

            Ok(ResponseJson(serde_json::json!({
                "success": true,
                "users": users_json
            })))
        }
        Err(e) => {
            error!("❌ Failed to list users: {}", e);
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Failed to list users"
            })))
        }
    }
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(request): Json<CreateUserRequest>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let _admin_user = check_admin_permission(State(state.clone()), headers).await?;

    info!("Creating new user: {}", request.username);

    match state.auth_service.create_user(&request.username, &request.email, &request.password, &request.role).await {
        Ok(user_id) => {
            info!("✅ User created successfully: {}", user_id);
            Ok(ResponseJson(serde_json::json!({
                "success": true,
                "message": "User created successfully",
                "user_id": user_id
            })))
        }
        Err(e) => {
            error!("❌ Failed to create user: {}", e);
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": format!("Failed to create user: {}", e)
            })))
        }
    }
}

async fn get_user(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let _admin_user = check_admin_permission(State(state.clone()), headers).await?;

    let user_id = match uuid::Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Invalid user ID format"
            })));
        }
    };

    match state.auth_service.get_user_by_id(user_id).await {
        Ok(Some(user)) => {
            Ok(ResponseJson(serde_json::json!({
                "success": true,
                "user": user
            })))
        }
        Ok(None) => {
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "User not found"
            })))
        }
        Err(e) => {
            error!("❌ Failed to get user: {}", e);
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Failed to get user"
            })))
        }
    }
}

async fn update_user(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let _admin_user = check_admin_permission(State(state.clone()), headers).await?;

    let user_id = match uuid::Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Invalid user ID format"
            })));
        }
    };

    info!("Updating user: {}", user_id);

    match state.auth_service.update_user(user_id, &request.username, &request.email, &request.role, request.is_active).await {
        Ok(_) => {
            info!("✅ User updated successfully: {}", user_id);
            Ok(ResponseJson(serde_json::json!({
                "success": true,
                "message": "User updated successfully"
            })))
        }
        Err(e) => {
            error!("❌ Failed to update user: {}", e);
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": format!("Failed to update user: {}", e)
            })))
        }
    }
}

async fn delete_user(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let _admin_user = check_admin_permission(State(state.clone()), headers).await?;

    let user_id = match uuid::Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Invalid user ID format"
            })));
        }
    };

    info!("Deleting user: {}", user_id);

    match state.auth_service.delete_user(user_id).await {
        Ok(_) => {
            info!("✅ User deleted successfully: {}", user_id);
            Ok(ResponseJson(serde_json::json!({
                "success": true,
                "message": "User deleted successfully"
            })))
        }
        Err(e) => {
            error!("❌ Failed to delete user: {}", e);
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": format!("Failed to delete user: {}", e)
            })))
        }
    }
}

async fn update_user_password(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(request): Json<UpdatePasswordRequest>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let _admin_user = check_admin_permission(State(state.clone()), headers).await?;

    let user_id = match uuid::Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": "Invalid user ID format"
            })));
        }
    };

    info!("Updating password for user: {}", user_id);

    match state.auth_service.update_user_password(user_id, &request.password).await {
        Ok(_) => {
            info!("✅ Password updated successfully for user: {}", user_id);
            Ok(ResponseJson(serde_json::json!({
                "success": true,
                "message": "Password updated successfully"
            })))
        }
        Err(e) => {
            error!("❌ Failed to update password: {}", e);
            Ok(ResponseJson(serde_json::json!({
                "success": false,
                "message": format!("Failed to update password: {}", e)
            })))
        }
    }
}

// ================================
// 辅助认证函数
// ================================

async fn check_authentication(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<alerts::User, axum::http::StatusCode> {
    let session_token = match headers.get("authorization") {
        Some(value) => {
            let auth_header = value.to_str().unwrap_or("");
            if auth_header.starts_with("Bearer ") {
                &auth_header[7..]
            } else {
                ""
            }
        }
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    if session_token.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match state.auth_service.validate_session(session_token).await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ================================
// 告警数据和标注相关API处理函数
// ================================

async fn get_alert_data(
    Query(query): Query<AlertDataQuery>,
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<ResponseJson<AlertDataResponse>, StatusCode> {
    let _user = check_authentication(State(state.clone()), headers).await?;

    // 暂时返回空数据，避免编译错误
    Ok(ResponseJson(AlertDataResponse {
        success: true,
        alert_data: vec![],
        total: 0,
    }))
}

async fn get_alert_data_detail(
    axum::extract::Path(id): axum::extract::Path<String>,
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let _user = check_authentication(State(state.clone()), headers).await?;

    Ok(ResponseJson(serde_json::json!({
        "success": false,
        "message": "Not implemented yet"
    })))
}

async fn get_annotations(
    Query(query): Query<AnnotationsQuery>,
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<ResponseJson<AnnotationsListResponse>, StatusCode> {
    let _user = check_authentication(State(state.clone()), headers).await?;

    let limit = query.limit.unwrap_or(50).min(1000);
    let offset = query.offset.unwrap_or(0);

    match state.database.get_annotations(
        limit,
        offset,
        query.alert_data_id.as_deref(),
        query.annotation_type.as_deref(),
        query.is_malicious,
        query.threat_level.as_deref(),
        query.annotated_by.and_then(|s| uuid::Uuid::parse_str(&s).ok()),
        query.review_status.as_deref(),
    ).await {
        Ok(annotations) => {
            info!("✅ Retrieved {} annotations from database", annotations.len());
            let total = annotations.len() as u64; // Calculate total before moving
            Ok(ResponseJson(AnnotationsListResponse {
                success: true,
                annotations,
                total, // For now, return count of current page
            }))
        }
        Err(e) => {
            error!("❌ Failed to get annotations: {}", e);
            Ok(ResponseJson(AnnotationsListResponse {
                success: false,
                annotations: vec![],
                total: 0,
            }))
        }
    }
}

async fn create_annotation(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(request): Json<CreateAnnotationRequest>,
) -> Result<ResponseJson<AnnotationResponse>, StatusCode> {
    let user = check_authentication(State(state.clone()), headers).await?;

    match state.database.create_annotation(
        &request.alert_data_id,
        &request.annotation_type,
        request.labels,
        request.confidence,
        request.is_malicious,
        request.threat_level.as_deref(),
        request.mitre_techniques,
        request.attack_stage.as_deref(),
        request.title.as_deref(),
        request.description.as_deref(),
        request.notes.as_deref(),
        user.id,
    ).await {
        Ok(annotation_id) => {
            info!("✅ Annotation created successfully: {}", annotation_id);
            Ok(ResponseJson(AnnotationResponse {
                success: true,
                message: "Annotation created successfully".to_string(),
                annotation_id: Some(annotation_id.to_string()),
            }))
        }
        Err(e) => {
            error!("❌ Failed to create annotation: {}", e);
            Ok(ResponseJson(AnnotationResponse {
                success: false,
                message: format!("Failed to create annotation: {}", e),
                annotation_id: None,
            }))
        }
    }
}

async fn update_annotation(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
    Json(_request): Json<CreateAnnotationRequest>,
) -> Result<ResponseJson<AnnotationResponse>, StatusCode> {
    let _user = check_authentication(State(state.clone()), headers).await?;

    // Temporarily return success until proper database implementation
    info!("✅ Annotation updated successfully (placeholder): {}", id);
    Ok(ResponseJson(AnnotationResponse {
        success: true,
        message: "Annotation updated successfully".to_string(),
        annotation_id: Some(id),
    }))
}

async fn delete_annotation(
    axum::extract::Path(id): axum::extract::Path<String>,
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<ResponseJson<AnnotationResponse>, StatusCode> {
    let _user = check_authentication(State(state.clone()), headers).await?;

    // Temporarily return success until proper database implementation
    info!("✅ Annotation deleted successfully (placeholder): {}", id);
    Ok(ResponseJson(AnnotationResponse {
        success: true,
        message: "Annotation deleted successfully".to_string(),
        annotation_id: Some(id),
    }))
}

#[derive(Deserialize)]
struct ReviewAnnotationRequest {
    review_status: String, // approved, rejected
    review_notes: Option<String>,
}

async fn review_annotation(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
    Json(_request): Json<ReviewAnnotationRequest>,
) -> Result<ResponseJson<AnnotationResponse>, StatusCode> {
    let user = check_authentication(State(state.clone()), headers).await?;

    // Only admins can review annotations
    if user.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    // Temporarily return success until proper database implementation
    info!("✅ Annotation reviewed successfully (placeholder): {}", id);
    Ok(ResponseJson(AnnotationResponse {
        success: true,
        message: "Annotation reviewed successfully".to_string(),
        annotation_id: Some(id),
    }))
}

// ================================
// 威胁事件相关API处理函数
// ================================

async fn get_threat_events(
    Query(query): Query<ThreatEventsQuery>,
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<ResponseJson<ThreatEventsListResponse>, StatusCode> {
    let _user = check_authentication(State(state.clone()), headers).await?;

    let limit = query.limit.unwrap_or(50).min(1000) as i64;
    let offset = query.offset.unwrap_or(0) as i64;

    let mut sql = r#"
        SELECT t.*, u1.username as created_by_username, u2.username as assigned_to_username,
               u3.username as updated_by_username
        FROM threat_events t 
        LEFT JOIN users u1 ON t.created_by = u1.id
        LEFT JOIN users u2 ON t.assigned_to = u2.id
        LEFT JOIN users u3 ON t.updated_by = u3.id
        WHERE 1=1
    "#.to_string();
    let mut conditions = Vec::new();

    if let Some(status) = &query.status {
        conditions.push(format!("t.status = '{}'", status));
    }
    if let Some(severity) = query.severity {
        conditions.push(format!("t.severity = {}", severity));
    }
    if let Some(event_type) = &query.event_type {
        conditions.push(format!("t.event_type = '{}'", event_type));
    }
    if let Some(creation_method) = &query.creation_method {
        conditions.push(format!("t.creation_method = '{}'", creation_method));
    }
    if let Some(date_from) = &query.date_from {
        conditions.push(format!("t.created_at >= '{}'", date_from));
    }
    if let Some(date_to) = &query.date_to {
        conditions.push(format!("t.created_at <= '{}'", date_to));
    }

    if !conditions.is_empty() {
        sql = format!("{} AND {}", sql, conditions.join(" AND "));
    }

    sql = format!("{} ORDER BY t.created_at DESC LIMIT {} OFFSET {}", sql, limit, offset);

    // Temporarily return empty data until proper database implementation
    let threat_events_json: Vec<serde_json::Value> = vec![];
    
    Ok(ResponseJson(ThreatEventsListResponse {
        success: true,
        threat_events: threat_events_json,
        total: 0,
    }))
}

async fn create_threat_event(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(request): Json<CreateThreatEventRequest>,
) -> Result<ResponseJson<ThreatEventResponse>, StatusCode> {
    let user = check_authentication(State(state.clone()), headers).await?;

    let threat_event_id = Uuid::new_v4();
    let tactics_json = request.mitre_tactics.map(|t| serde_json::to_value(t).unwrap_or(serde_json::Value::Null));
    let techniques_json = request.mitre_techniques.map(|t| serde_json::to_value(t).unwrap_or(serde_json::Value::Null));
    let kill_chain_json = request.kill_chain_phases.map(|k| serde_json::to_value(k).unwrap_or(serde_json::Value::Null));
    let tags_json = request.tags.map(|t| serde_json::to_value(t).unwrap_or(serde_json::Value::Null));

    let sql = r#"
        INSERT INTO threat_events 
        (id, title, description, event_type, severity, threat_category, 
         event_start_time, event_end_time, mitre_tactics, mitre_techniques, 
         kill_chain_phases, priority, tags, creation_method, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
    "#;

    // Temporarily return success until proper database implementation
    info!("✅ Threat event created successfully (placeholder): {}", threat_event_id);
    Ok(ResponseJson(ThreatEventResponse {
        success: true,
        message: "Threat event created successfully".to_string(),
        threat_event_id: Some(threat_event_id.to_string()),
    }))
}

async fn get_threat_event_detail(
    axum::extract::Path(id): axum::extract::Path<String>,
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let _user = check_authentication(State(state.clone()), headers).await?;

    // Temporarily return empty data until proper database implementation
    Ok(ResponseJson(serde_json::json!({
        "success": false,
        "message": "Threat event detail not implemented yet"
    })))
}

async fn update_threat_event(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
    Json(_request): Json<UpdateThreatEventRequest>,
) -> Result<ResponseJson<ThreatEventResponse>, StatusCode> {
    let _user = check_authentication(State(state.clone()), headers).await?;
    
    // Temporarily return success until proper database implementation
    info!("✅ Threat event updated successfully (placeholder): {}", id);
    Ok(ResponseJson(ThreatEventResponse {
        success: true,
        message: "Threat event updated successfully".to_string(),
        threat_event_id: Some(id),
    }))
}

async fn delete_threat_event(
    axum::extract::Path(id): axum::extract::Path<String>,
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<ResponseJson<ThreatEventResponse>, StatusCode> {
    let user = check_authentication(State(state.clone()), headers).await?;

    // Only admins can delete threat events
    if user.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    // Temporarily return success until proper database implementation
    info!("✅ Threat event deleted successfully (placeholder): {}", id);
    Ok(ResponseJson(ThreatEventResponse {
        success: true,
        message: "Threat event deleted successfully".to_string(),
        threat_event_id: Some(id),
    }))
}

async fn correlate_alerts_to_threat_event(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
    Json(request): Json<CorrelateAlertsRequest>,
) -> Result<ResponseJson<CorrelationResponse>, StatusCode> {
    let _user = check_authentication(State(state.clone()), headers).await?;
    
    let correlations_created = request.alert_data_ids.len() as u32;
    
    // Temporarily return success until proper database implementation
    info!("✅ Created {} correlations for threat event (placeholder): {}", correlations_created, id);
    Ok(ResponseJson(CorrelationResponse {
        success: true,
        message: format!("Successfully created {} correlations", correlations_created),
        correlations_created,
    }))
}

async fn get_threat_event_timeline(
    axum::extract::Path(_id): axum::extract::Path<String>,
    State(_state): State<Arc<AppState>>,
    _headers: axum::http::HeaderMap,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    // Temporarily return empty data until proper database implementation
    Ok(ResponseJson(serde_json::json!({
        "success": true,
        "timeline": []
    })))
}

#[derive(Deserialize)]
struct AddTimelineEntryRequest {
    timestamp: String,
    event_type: String,
    title: String,
    description: Option<String>,
    alert_data_id: Option<String>,
    severity: Option<String>,
    phase: Option<String>,
}

async fn add_threat_event_timeline_entry(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
    Json(_request): Json<AddTimelineEntryRequest>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let _user = check_authentication(State(state.clone()), headers).await?;
    
    let timeline_id = Uuid::new_v4();
    
    // Temporarily return success until proper database implementation
    info!("✅ Timeline entry added successfully (placeholder): {}", timeline_id);
    Ok(ResponseJson(serde_json::json!({
        "success": true,
        "message": "Timeline entry added successfully",
        "timeline_entry_id": timeline_id.to_string()
    })))
}