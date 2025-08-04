import React, { useState, useEffect } from 'react';
import { LineChart, Line, AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, BarChart, Bar } from 'recharts';
import axios from 'axios';

const KafkaDataChart = () => {
  const [chartData, setChartData] = useState([]);
  const [dataType, setDataType] = useState('rate'); // rate, types, severity
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    // Fetch initial data
    fetchKafkaStats();
    
    // Set up polling for real-time updates
    const interval = setInterval(() => {
      fetchKafkaStats();
    }, 5000); // Update every 5 seconds

    return () => clearInterval(interval);
  }, []);

  const fetchKafkaStats = async () => {
    try {
      const response = await axios.get('/api/kafka-stats');
      const stats = response.data;
      
      // Process data for the chart
      console.log('Received stats:', stats); // Debug log
      const newDataPoint = {
        time: new Date().toLocaleTimeString(),
        timestamp: Date.now(),
        messageRate: stats.message_rate || 0,
        totalMessages: stats.total_messages || 0,
        edrCount: stats.type_breakdown?.edr || 0,
        ngavCount: stats.type_breakdown?.ngav || 0,
        alertCount: stats.type_breakdown?.alert || 0,
        criticalCount: stats.severity_breakdown?.critical || 0,
        warningCount: stats.severity_breakdown?.warning || 0,
        infoCount: stats.severity_breakdown?.info || 0,
      };

      setChartData(prevData => {
        const newData = [...prevData, newDataPoint];
        // Keep only last 20 data points
        if (newData.length > 20) {
          newData.shift();
        }
        return newData;
      });
      
      setIsLoading(false);
      setError(null);
    } catch (err) {
      console.error('Failed to fetch Kafka stats:', err);
      setError('无法获取Kafka数据统计');
      setIsLoading(false);
    }
  };

  const renderChart = () => {
    if (chartData.length === 0) {
      return <div className="chart-placeholder">等待数据...</div>;
    }

    switch (dataType) {
      case 'rate':
        return (
          <ResponsiveContainer width="100%" height={400}>
            <AreaChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="time" />
              <YAxis />
              <Tooltip />
              <Legend />
              <Area 
                type="monotone" 
                dataKey="messageRate" 
                stroke="#3B82F6" 
                fill="#93C5FD" 
                name="消息速率 (条/秒)"
              />
            </AreaChart>
          </ResponsiveContainer>
        );
      
      case 'types':
        return (
          <ResponsiveContainer width="100%" height={400}>
            <LineChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="time" />
              <YAxis />
              <Tooltip />
              <Legend />
              <Line type="monotone" dataKey="edrCount" stroke="#10B981" name="EDR告警" strokeWidth={2} />
              <Line type="monotone" dataKey="ngavCount" stroke="#F59E0B" name="NGAV告警" strokeWidth={2} />
              <Line type="monotone" dataKey="alertCount" stroke="#6366F1" name="通用告警" strokeWidth={2} />
            </LineChart>
          </ResponsiveContainer>
        );
      
      case 'severity':
        return (
          <ResponsiveContainer width="100%" height={400}>
            <BarChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="time" />
              <YAxis />
              <Tooltip />
              <Legend />
              <Bar dataKey="criticalCount" stackId="a" fill="#EF4444" name="严重" />
              <Bar dataKey="warningCount" stackId="a" fill="#F59E0B" name="警告" />
              <Bar dataKey="infoCount" stackId="a" fill="#3B82F6" name="信息" />
            </BarChart>
          </ResponsiveContainer>
        );
      
      default:
        return null;
    }
  };

  if (isLoading && chartData.length === 0) {
    return (
      <div className="chart-container">
        <div className="chart-loading">加载中...</div>
      </div>
    );
  }

  if (error && chartData.length === 0) {
    return (
      <div className="chart-container">
        <div className="chart-error">{error}</div>
      </div>
    );
  }

  return (
    <div className="kafka-chart-container">
      <div className="chart-header">
        <h3>📊 Kafka 实时数据监控</h3>
        <div className="chart-controls">
          <button 
            className={`chart-type-btn ${dataType === 'rate' ? 'active' : ''}`}
            onClick={() => setDataType('rate')}
          >
            消息速率
          </button>
          <button 
            className={`chart-type-btn ${dataType === 'types' ? 'active' : ''}`}
            onClick={() => setDataType('types')}
          >
            告警类型
          </button>
          <button 
            className={`chart-type-btn ${dataType === 'severity' ? 'active' : ''}`}
            onClick={() => setDataType('severity')}
          >
            严重程度
          </button>
        </div>
      </div>
      
      <div className="chart-wrapper">
        {renderChart()}
      </div>
      
      <div className="chart-stats">
        {chartData.length > 0 && (
          <>
            <div className="stat-item">
              <span className="stat-label">当前速率:</span>
              <span className="stat-value">{chartData[chartData.length - 1].messageRate} 条/秒</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">总消息数:</span>
              <span className="stat-value">{chartData[chartData.length - 1].totalMessages.toLocaleString()}</span>
            </div>
          </>
        )}
      </div>
    </div>
  );
};

export default KafkaDataChart;