#!/bin/bash

# Neo4j图数据库启动脚本
# 用于告警关联图谱功能

echo "🚀 启动Neo4j图数据库..."

# 检查Docker是否安装
if ! command -v docker &> /dev/null; then
    echo "❌ Docker未安装，请先安装Docker"
    exit 1
fi

# 检查Docker服务是否运行
if ! docker info &> /dev/null; then
    echo "❌ Docker服务未运行，请启动Docker服务"
    exit 1
fi

# 创建必要的目录
echo "📁 创建数据目录..."
mkdir -p ./docker/neo4j/{data,logs,import,plugins}

# 停止已存在的容器
echo "🛑 停止已存在的Neo4j容器..."
docker-compose -f docker/docker-compose.neo4j.yml down 2>/dev/null || true

# 启动Neo4j
echo "🔧 启动Neo4j容器..."
docker-compose -f docker/docker-compose.neo4j.yml up -d

# 等待Neo4j启动
echo "⏳ 等待Neo4j启动..."
sleep 10

# 检查Neo4j状态
if docker ps | grep -q neo4j-alerts; then
    echo "✅ Neo4j启动成功！"
    echo ""
    echo "📊 Neo4j访问信息："
    echo "  - Web界面: http://localhost:7474"
    echo "  - Bolt连接: bolt://localhost:7687"
    echo "  - 用户名: neo4j"
    echo "  - 密码: alerts123"
    echo ""
    echo "💡 提示："
    echo "  1. 首次访问Web界面可能需要等待几秒钟"
    echo "  2. 使用 'docker logs neo4j-alerts' 查看日志"
    echo "  3. 使用 'docker-compose -f docker/docker-compose.neo4j.yml down' 停止服务"
else
    echo "❌ Neo4j启动失败，请检查日志："
    echo "docker logs neo4j-alerts"
    exit 1
fi