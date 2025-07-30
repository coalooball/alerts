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

    // 选择发送模式：示例告警或EDR数据
    let edr_file_path = "atlasv2/data/attack/h1/cbc-edr-alerts/edr-alerts-h1-m1.jsonl";
    
    if std::path::Path::new(edr_file_path).exists() {
        info!("📊 Loading and sending EDR alerts from file...");
        match producer.load_and_send_jsonl_file(edr_file_path).await {
            Ok(count) => info!("✅ Successfully sent {} EDR alerts", count),
            Err(e) => {
                log::error!("Failed to load EDR file: {}", e);
                info!("📤 Falling back to sample alerts...");
                send_sample_alerts(&producer).await?;
            }
        }
    } else {
        info!("📤 EDR file not found, sending sample alerts...");
        send_sample_alerts(&producer).await?;
    }

    // 等待消费者处理消息
    tokio::time::sleep(Duration::from_secs(10)).await;

    info!("✅ Application completed successfully");

    Ok(())
}
