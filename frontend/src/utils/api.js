import axios from 'axios';

export const api = {
  // Kafka配置相关
  getConfigs: () => axios.get('/api/configs'),
  getActiveConfigs: () => axios.get('/api/configs/active'),
  createConfig: (config) => axios.post('/api/config', config),
  updateConfig: (id, config) => axios.put(`/api/config/${id}`, config),
  deleteConfig: (id) => axios.delete(`/api/config/${id}`),
  toggleConfig: (id, isActive) => axios.post(`/api/config/${id}/toggle`, { is_active: isActive }),
  
  // ClickHouse配置相关
  getClickhouseConfig: () => axios.get('/api/clickhouse-config'),
  saveClickhouseConfig: (config) => axios.post('/api/clickhouse-config', config),
  
  // 连接测试相关
  testKafkaConnectivity: (bootstrapServers, topic) => 
    axios.get('/api/test-connectivity', {
      params: { bootstrap_servers: bootstrapServers, topic }
    }),
  testClickhouseConnectivity: () => axios.get('/api/test-clickhouse-connectivity'),
  
  // 消息相关
  consumeMessages: () => axios.get('/api/consume-messages')
};

export default api;