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

    // 选择发送模式：自动检测文件格式或示例告警
    let file_paths = vec![
        "atlasv2/data/attack/h1/cbc-edr-alerts/edr-alerts-h1-m1.jsonl",
        "atlasv2/data/attack/h1/cbc-ngav-alerts/ngav-alerts-h1-m2.jsonl",
    ];
    
    let mut files_processed = false;
    
    for file_path in file_paths {
        if std::path::Path::new(file_path).exists() {
            info!("📊 Auto-detecting and loading alerts from: {}", file_path);
            match producer.detect_and_load_file(file_path).await {
                Ok(count) => {
                    info!("✅ Successfully sent {} alerts from {}", count, file_path);
                    files_processed = true;
                }
                Err(e) => {
                    log::error!("Failed to load file {}: {}", file_path, e);
                }
            }
            
            // 在文件之间等待一下
            tokio::time::sleep(Duration::from_secs(2)).await;
        } else {
            info!("📂 File not found: {}", file_path);
        }
    }
    
    if !files_processed {
        info!("📤 No alert files found, sending sample alerts...");
        send_sample_alerts(&producer).await?;
    }

    // 等待消费者处理消息
    tokio::time::sleep(Duration::from_secs(10)).await;

    info!("✅ Application completed successfully");

    Ok(())
}
