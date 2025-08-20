use axum::{
    extract::{Path, Query, State},
    response::Json,
    http::StatusCode,
    Json as JsonExtractor,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::neo4j_client::{Neo4jClient, Neo4jConfig, GraphVisualizationData};

// API请求/响应结构
#[derive(Debug, Deserialize)]
pub struct GetGraphQuery {
    pub depth: Option<i64>,
    pub max_nodes: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct CorrelateAlertsRequest {
    pub time_window_hours: i64,
}

#[derive(Debug, Deserialize)]
pub struct LateralMovementQuery {
    pub org_key: String,
    pub hours: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg),
        }
    }
}

// 图谱服务状态
pub struct GraphApiState {
    pub neo4j_client: Arc<Neo4jClient>,
}

// 获取告警关联图谱
pub async fn get_alert_graph(
    Path(alert_id): Path<String>,
    Query(params): Query<GetGraphQuery>,
    State(state): State<Arc<GraphApiState>>,
) -> Result<Json<ApiResponse<GraphVisualizationData>>, StatusCode> {
    let depth = params.depth.unwrap_or(2);
    
    match state.neo4j_client.get_alert_graph(&alert_id, depth).await {
        Ok(graph_data) => Ok(Json(ApiResponse::success(graph_data))),
        Err(e) => {
            eprintln!("Error getting alert graph: {}", e);
            Ok(Json(ApiResponse::error(format!("Failed to get graph: {}", e))))
        }
    }
}

// 自动关联告警
pub async fn correlate_alerts(
    State(state): State<Arc<GraphApiState>>,
    JsonExtractor(req): JsonExtractor<CorrelateAlertsRequest>,
) -> Result<Json<ApiResponse<i64>>, StatusCode> {
    match state.neo4j_client.correlate_alerts(req.time_window_hours).await {
        Ok(count) => Ok(Json(ApiResponse::success(count))),
        Err(e) => {
            eprintln!("Error correlating alerts: {}", e);
            Ok(Json(ApiResponse::error(format!("Failed to correlate: {}", e))))
        }
    }
}

// 检测横向移动
pub async fn detect_lateral_movement(
    Query(params): Query<LateralMovementQuery>,
    State(state): State<Arc<GraphApiState>>,
) -> Result<Json<ApiResponse<Vec<String>>>, StatusCode> {
    let hours = params.hours.unwrap_or(24);
    
    match state.neo4j_client.detect_lateral_movement(&params.org_key, hours).await {
        Ok(movements) => Ok(Json(ApiResponse::success(movements))),
        Err(e) => {
            eprintln!("Error detecting lateral movement: {}", e);
            Ok(Json(ApiResponse::error(format!("Detection failed: {}", e))))
        }
    }
}

// 初始化Neo4j数据库
pub async fn init_neo4j_db(
    State(state): State<Arc<GraphApiState>>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match state.neo4j_client.init_database().await {
        Ok(_) => Ok(Json(ApiResponse::success("Database initialized".to_string()))),
        Err(e) => {
            eprintln!("Error initializing database: {}", e);
            Ok(Json(ApiResponse::error(format!("Initialization failed: {}", e))))
        }
    }
}

// 清理过期数据
pub async fn cleanup_old_data(
    State(state): State<Arc<GraphApiState>>,
) -> Result<Json<ApiResponse<i64>>, StatusCode> {
    match state.neo4j_client.cleanup_old_data(30).await {
        Ok(count) => Ok(Json(ApiResponse::success(count))),
        Err(e) => {
            eprintln!("Error cleaning up data: {}", e);
            Ok(Json(ApiResponse::error(format!("Cleanup failed: {}", e))))
        }
    }
}

// 创建图谱路由
pub fn create_graph_routes() -> axum::Router<Arc<GraphApiState>> {
    use axum::routing::{get, post};
    
    axum::Router::new()
        .route("/api/graph/alert/:id", get(get_alert_graph))
        .route("/api/graph/correlate", post(correlate_alerts))
        .route("/api/graph/lateral-movement", get(detect_lateral_movement))
        .route("/api/graph/init", post(init_neo4j_db))
        .route("/api/graph/cleanup", post(cleanup_old_data))
}