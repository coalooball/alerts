use anyhow::Result;
use log::info;
use std::time::Duration;

use alerts::{AlertMessage, KafkaConfig, KafkaConsumer, KafkaProducer};

// 示例：发送告警消息
async fn send_sample_alerts(producer: &KafkaProducer) -> Result<()> {
    let alerts = vec![
        AlertMessage::critical(
            "alert-001".to_string(),
            "Database connection failed - all services affected".to_string(),
        ),
        AlertMessage::warning(
            "alert-002".to_string(),
            "High memory usage detected (85%)".to_string(),
        ),
        AlertMessage::info(
            "alert-003".to_string(),
            "Service started successfully".to_string(),
        ),
        AlertMessage::critical(
            "alert-004".to_string(),
            "API rate limit exceeded - service degraded".to_string(),
        ),
    ];

    producer.send_batch(alerts).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();

    info!("🚀 Starting Kafka alerts application...");

    let config = KafkaConfig::from_file("config.toml")
        .unwrap_or_else(|e| {
            log::warn!("Failed to load config.toml: {}. Using default configuration.", e);
            KafkaConfig::default()
        });

    // 创建生产者和消费者
    let producer = KafkaProducer::new(config.clone()).await?;
    let consumer = KafkaConsumer::new(config).await?;

    // 启动消费者任务
    let _consumer_handle = tokio::spawn(async move {
        if let Err(e) = consumer.consume_alerts().await {
            log::error!("Consumer error: {}", e);
        }
    });

    // 等待一下让消费者启动
    tokio::time::sleep(Duration::from_secs(2)).await;

    // 发送示例告警
    info!("📤 Sending sample alerts...");
    send_sample_alerts(&producer).await?;

    // 等待消费者处理消息
    tokio::time::sleep(Duration::from_secs(5)).await;

    info!("✅ Application completed successfully");

    Ok(())
}
