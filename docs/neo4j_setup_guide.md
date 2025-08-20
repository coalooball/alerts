# Neo4j 图数据库集成指南

## 🎯 功能概述

本系统集成了Neo4j图数据库，用于实现告警数据的关联分析和可视化，主要功能包括：

- **告警关联图谱**：可视化展示告警之间的关联关系
- **攻击链分析**：追踪完整的攻击路径
- **横向移动检测**：自动识别内网横向移动行为
- **IOC关联**：基于威胁指标关联相关告警
- **时序分析**：基于时间窗口自动关联告警

## 🚀 快速开始

### 1. 启动Neo4j数据库

```bash
# 方式1：使用提供的脚本
chmod +x scripts/start_neo4j.sh
./scripts/start_neo4j.sh

# 方式2：直接使用docker-compose
docker-compose -f docker/docker-compose.neo4j.yml up -d
```

### 2. 验证Neo4j运行状态

```bash
# 检查容器状态
docker ps | grep neo4j-alerts

# 查看日志
docker logs neo4j-alerts
```

### 3. 访问Neo4j界面

打开浏览器访问: http://localhost:7474
- 用户名: `neo4j`
- 密码: `alerts123`

### 4. 初始化数据库

```bash
# 通过API初始化数据库结构
curl -X POST http://localhost:3000/api/graph/init \
  -H "Authorization: Bearer YOUR_SESSION_TOKEN"
```

## 📊 使用图谱功能

### 在Web界面中使用

1. 登录系统后，进入"告警分析"页面
2. 选择一个告警，点击"查看关联图谱"
3. 使用图谱控制面板：
   - 调整展示深度（1-5层）
   - 切换布局模式（层级/物理）
   - 双击节点展开关联
   - 点击"自动关联"建立告警关系

### API接口使用

#### 获取告警图谱
```bash
GET /api/graph/alert/{alert_id}?depth=2
```

#### 自动关联告警
```bash
POST /api/graph/correlate
{
  "time_window_hours": 24
}
```

#### 检测横向移动
```bash
GET /api/graph/lateral-movement?org_key=default&hours=24
```

## 🔧 配置说明

### Neo4j配置

编辑 `docker/docker-compose.neo4j.yml` 调整配置：

```yaml
environment:
  # 内存配置（根据服务器资源调整）
  - NEO4J_dbms_memory_pagecache_size=2G  # 页面缓存
  - NEO4J_dbms_memory_heap_max__size=2G  # JVM堆内存
```

### Rust客户端配置

在代码中配置Neo4j连接：

```rust
let config = Neo4jConfig {
    uri: "bolt://localhost:7687".to_string(),
    username: "neo4j".to_string(),
    password: "alerts123".to_string(),
    database: "alerts".to_string(),
};
```

## 📈 性能优化

### 1. 索引优化

系统自动创建以下索引：
- Alert节点: id, severity, create_time, alert_type
- Device节点: device_id, device_name
- Process节点: process_guid, process_path

### 2. 查询优化

- 使用参数化查询避免查询计划重编译
- 限制图遍历深度（建议不超过5层）
- 使用投影只返回需要的字段

### 3. 数据清理

定期清理过期数据：

```bash
# 清理30天前的数据
curl -X POST http://localhost:3000/api/graph/cleanup \
  -H "Authorization: Bearer YOUR_SESSION_TOKEN"
```

## 🎨 图谱可视化说明

### 节点颜色含义

- **红色**：严重告警（severity >= 8）
- **橙色**：高危告警（severity >= 6）
- **黄色**：中危告警（severity >= 4）
- **绿色**：低危告警（severity < 4）
- **蓝色**：设备节点
- **紫色**：IOC节点
- **棕色**：用户节点

### 边类型说明

- `TRIGGERED_ON`：告警触发于设备
- `INVOLVES_PROCESS`：告警涉及进程
- `PARENT_OF`：父子进程关系
- `CORRELATED_WITH`：告警关联关系
- `CONTAINS_IOC`：包含威胁指标

## 🛠️ 故障排查

### Neo4j无法启动

```bash
# 检查端口占用
lsof -i :7474
lsof -i :7687

# 清理旧数据重新启动
docker-compose -f docker/docker-compose.neo4j.yml down -v
rm -rf docker/neo4j/data/*
docker-compose -f docker/docker-compose.neo4j.yml up -d
```

### 连接失败

```bash
# 测试Neo4j连接
curl http://localhost:7474

# 检查防火墙
sudo ufw status
```

### 性能问题

```bash
# 进入Neo4j shell检查
docker exec -it neo4j-alerts cypher-shell -u neo4j -p alerts123

# 查看数据库统计
CALL db.stats.retrieve("GRAPH");

# 查看慢查询
CALL dbms.listQueries() YIELD query, elapsedTimeMillis 
WHERE elapsedTimeMillis > 1000 
RETURN query, elapsedTimeMillis;
```

## 📚 Cypher查询示例

### 查找最活跃的攻击源
```cypher
MATCH (a:Alert)-[:TRIGGERED_ON]->(d:Device)
RETURN d.device_name, count(a) as alert_count
ORDER BY alert_count DESC
LIMIT 10
```

### 查找攻击路径
```cypher
MATCH path = (start:Alert)-[:CORRELATED_WITH*1..5]->(end:Alert)
WHERE start.severity > 7
RETURN path
LIMIT 20
```

### 查找共同IOC
```cypher
MATCH (a1:Alert)-[:CONTAINS_IOC]->(ioc:IOC)<-[:CONTAINS_IOC]-(a2:Alert)
WHERE a1.id <> a2.id
RETURN a1, ioc, a2
```

## 🔗 相关资源

- [Neo4j官方文档](https://neo4j.com/docs/)
- [Cypher查询语言](https://neo4j.com/docs/cypher-manual/)
- [vis.js网络图文档](https://visjs.github.io/vis-network/docs/network/)

## 📝 注意事项

1. Neo4j社区版限制：
   - 单个数据库
   - 无集群支持
   - 建议数据量 < 1000万节点

2. 生产环境建议：
   - 使用企业版Neo4j
   - 配置SSL/TLS加密
   - 设置强密码
   - 定期备份数据

3. 数据隐私：
   - 图数据库中包含敏感告警信息
   - 确保访问控制
   - 定期审计访问日志