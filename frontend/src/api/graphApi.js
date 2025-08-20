import axios from 'axios';

const API_BASE = '/api';

// 创建带认证的axios实例
const api = axios.create({
  baseURL: API_BASE,
  headers: {
    'Content-Type': 'application/json'
  }
});

// 添加认证拦截器
api.interceptors.request.use(config => {
  const token = localStorage.getItem('sessionToken');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// 图谱相关API
export const graphApi = {
  // 初始化图数据库
  initDatabase: async () => {
    const response = await api.post('/graph/init');
    return response.data;
  },

  // 获取告警图谱
  getAlertGraph: async (alertId, depth = 2) => {
    const response = await api.get(`/graph/alert/${alertId}`, {
      params: { depth }
    });
    return response.data;
  },

  // 自动关联告警
  correlateAlerts: async (timeWindowHours = 24) => {
    const response = await api.post('/graph/correlate', {
      time_window_hours: timeWindowHours
    });
    return response.data;
  },

  // 检测横向移动
  detectLateralMovement: async (orgKey = 'default', hours = 24) => {
    const response = await api.get('/graph/lateral-movement', {
      params: { org_key: orgKey, hours }
    });
    return response.data;
  },

  // 清理旧数据
  cleanupOldData: async () => {
    const response = await api.post('/graph/cleanup');
    return response.data;
  },

  // 批量导入告警到图数据库
  importAlerts: async (alerts) => {
    const response = await api.post('/graph/import-alerts', { alerts });
    return response.data;
  }
};

// 告警数据API
export const alertApi = {
  // 获取告警列表
  getAlerts: async (params = {}) => {
    const response = await api.get('/alerts', { params });
    return response.data;
  },

  // 获取单个告警详情
  getAlertDetail: async (alertId) => {
    const response = await api.get(`/alerts/${alertId}`);
    return response.data;
  },

  // 从ClickHouse获取告警
  getClickHouseAlerts: async (limit = 100, offset = 0) => {
    const response = await api.get('/clickhouse/alerts', {
      params: { limit, offset }
    });
    return response.data;
  }
};

export default { graphApi, alertApi };