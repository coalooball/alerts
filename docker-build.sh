#!/bin/bash

# 构建 Docker 镜像
echo "Building Docker image..."
docker-compose build

echo "Build complete!"
echo ""
echo "To run the web server on port 3005:"
echo "  docker-compose up -d"
echo ""
echo "To initialize/reset the database:"
echo "  docker-compose -f docker-compose.init.yml up"
echo ""
echo "To view logs:"
echo "  docker-compose logs -f"
echo ""
echo "To stop the service:"
echo "  docker-compose down"