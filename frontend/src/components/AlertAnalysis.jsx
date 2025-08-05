import React, { useState, useEffect } from 'react';
import { PieChart, Pie, Cell, BarChart, Bar, LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import axios from 'axios';

const AlertAnalysis = () => {
  const [timeRange, setTimeRange] = useState('1h'); // 默认1小时
  const [severityData, setSeverityData] = useState([]);
  const [trendData, setTrendData] = useState([]);
  const [typeClusterData, setTypeClusterData] = useState([]);
  const [loading, setLoading] = useState(false);

  // 时间范围选项
  const timeRangeOptions = [
    { value: '15m', label: '15分钟' },
    { value: '30m', label: '30分钟' },
    { value: '1h', label: '1小时' },
    { value: '3h', label: '3小时' },
    { value: '6h', label: '6小时' },
    { value: '12h', label: '12小时' },
    { value: '24h', label: '24小时' },
    { value: '3d', label: '3天' },
    { value: '7d', label: '7天' }
  ];

  // 颜色配置
  const COLORS = {
    severity: ['#ff4444', '#ff8800', '#ffcc00', '#00aa00', '#0088cc'],
    trend: '#8884d8',
    cluster: ['#82ca9d', '#8dd1e1', '#d084d0', '#ffb347', '#87ceeb']
  };

  // 获取分析数据
  const fetchAnalysisData = async () => {
    setLoading(true);
    try {
      const [severityRes, trendRes, clusterRes] = await Promise.all([
        axios.get(`/api/analysis/severity-distribution?timeRange=${timeRange}`),
        axios.get(`/api/analysis/time-series?timeRange=${timeRange}`),
        axios.get(`/api/analysis/type-clustering?timeRange=${timeRange}`)
      ]);

      setSeverityData(severityRes.data);
      setTrendData(trendRes.data);
      setTypeClusterData(clusterRes.data);
    } catch (error) {
      console.error('获取分析数据失败:', error);
      // 使用模拟数据作为fallback
      setSeverityData([
        { name: '严重', value: 45, color: '#ff4444' },
        { name: '高危', value: 128, color: '#ff8800' },
        { name: '中危', value: 234, color: '#ffcc00' },
        { name: '低危', value: 567, color: '#00aa00' },
        { name: '信息', value: 89, color: '#0088cc' }
      ]);
      
      setTrendData([
        { time: '00:00', alerts: 12 },
        { time: '00:15', alerts: 19 },
        { time: '00:30', alerts: 8 },
        { time: '00:45', alerts: 25 },
        { time: '01:00', alerts: 34 }
      ]);
      
      setTypeClusterData([
        { type: 'EDR告警', count: 456, percentage: 45.2 },
        { type: 'NGAV告警', count: 234, percentage: 23.1 },
        { type: 'DNS异常', count: 123, percentage: 12.2 },
        { type: 'Sysmon事件', count: 98, percentage: 9.7 },
        { type: '其他', count: 98, percentage: 9.8 }
      ]);
    }
    setLoading(false);
  };

  useEffect(() => {
    fetchAnalysisData();
  }, [timeRange]);

  // 自定义标签渲染
  const renderCustomizedLabel = ({ cx, cy, midAngle, innerRadius, outerRadius, percent }) => {
    if (percent < 0.05) return null; // 小于5%不显示标签
    const RADIAN = Math.PI / 180;
    const radius = innerRadius + (outerRadius - innerRadius) * 0.5;
    const x = cx + radius * Math.cos(-midAngle * RADIAN);
    const y = cy + radius * Math.sin(-midAngle * RADIAN);

    return (
      <text 
        x={x} 
        y={y} 
        fill="white" 
        textAnchor={x > cx ? 'start' : 'end'} 
        dominantBaseline="central"
        fontSize="12"
        fontWeight="bold"
      >
        {`${(percent * 100).toFixed(1)}%`}
      </text>
    );
  };

  return (
    <div className="alert-analysis">
      <div className="analysis-header">
        <h2>📊 告警分析</h2>
        <div className="time-range-selector">
          <label>时间范围：</label>
          <select 
            value={timeRange} 
            onChange={(e) => setTimeRange(e.target.value)}
            className="time-range-select"
          >
            {timeRangeOptions.map(option => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
          <button onClick={fetchAnalysisData} className="refresh-btn" disabled={loading}>
            {loading ? '🔄' : '🔃'} 刷新
          </button>
        </div>
      </div>

      <div className="analysis-panels">
        {/* 告警严重程度分布 */}
        <div className="analysis-panel">
          <div className="panel-header">
            <h3>🎯 严重程度分布</h3>
            <span className="panel-subtitle">按告警严重程度统计</span>
          </div>
          <div className="chart-container">
            <ResponsiveContainer width="100%" height={300}>
              <PieChart>
                <Pie
                  data={severityData}
                  cx="50%"
                  cy="50%"
                  labelLine={false}
                  label={renderCustomizedLabel}
                  outerRadius={80}
                  fill="#8884d8"
                  dataKey="value"
                >
                  {severityData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color || COLORS.severity[index % COLORS.severity.length]} />
                  ))}
                </Pie>
                <Tooltip formatter={(value, name) => [value, name]} />
                <Legend />
              </PieChart>
            </ResponsiveContainer>
          </div>
          <div className="panel-stats">
            <div className="stat-item">
              <span className="stat-label">总告警数:</span>
              <span className="stat-value">{severityData.reduce((sum, item) => sum + item.value, 0)}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">高危及以上:</span>
              <span className="stat-value critical">
                {severityData.slice(0, 2).reduce((sum, item) => sum + item.value, 0)}
              </span>
            </div>
          </div>
        </div>

        {/* 时间序列趋势 */}
        <div className="analysis-panel">
          <div className="panel-header">
            <h3>📈 时间序列趋势</h3>
            <span className="panel-subtitle">告警数量时间分布</span>
          </div>
          <div className="chart-container">
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={trendData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="time" />
                <YAxis />
                <Tooltip />
                <Legend />
                <Line 
                  type="monotone" 
                  dataKey="alerts" 
                  stroke={COLORS.trend} 
                  strokeWidth={3}
                  dot={{ fill: COLORS.trend, strokeWidth: 2, r: 4 }}
                  activeDot={{ r: 6 }}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
          <div className="panel-stats">
            <div className="stat-item">
              <span className="stat-label">平均速率:</span>
              <span className="stat-value">
                {trendData.length > 0 ? 
                  Math.round(trendData.reduce((sum, item) => sum + item.alerts, 0) / trendData.length) 
                  : 0
                } 条/时段
              </span>
            </div>
            <div className="stat-item">
              <span className="stat-label">峰值:</span>
              <span className="stat-value warning">
                {Math.max(...trendData.map(item => item.alerts), 0)} 条
              </span>
            </div>
          </div>
        </div>

        {/* 告警类型聚类 */}
        <div className="analysis-panel">
          <div className="panel-header">
            <h3>🔍 告警类型聚类</h3>
            <span className="panel-subtitle">按告警类型分类统计</span>
          </div>
          <div className="chart-container">
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={typeClusterData} layout="horizontal">
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis type="number" />
                <YAxis dataKey="type" type="category" width={80} />
                <Tooltip />
                <Legend />
                <Bar 
                  dataKey="count" 
                  fill="#82ca9d"
                  name="告警数量"
                >
                  {typeClusterData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={COLORS.cluster[index % COLORS.cluster.length]} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
          <div className="panel-stats">
            <div className="stat-item">
              <span className="stat-label">类型数量:</span>
              <span className="stat-value">{typeClusterData.length}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">主要类型:</span>
              <span className="stat-value info">
                {typeClusterData.length > 0 ? typeClusterData[0].type : 'N/A'}
              </span>
            </div>
          </div>
        </div>
      </div>

      {loading && (
        <div className="loading-overlay">
          <div className="loading-spinner">🔄 正在加载分析数据...</div>
        </div>
      )}
    </div>
  );
};

export default AlertAnalysis;