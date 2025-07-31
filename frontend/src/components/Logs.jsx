import React, { useState, useEffect, useRef } from 'react';
import '../App.css';

const Logs = () => {
  const [activeTab, setActiveTab] = useState('monitoring');
  const [liveMessages, setLiveMessages] = useState([]);
  const [consumerStatus, setConsumerStatus] = useState({});
  const [isMonitoring, setIsMonitoring] = useState(false);
  const [maxMessages, setMaxMessages] = useState(100);
  const messagesEndRef = useRef(null);
  const intervalRef = useRef(null);

  useEffect(() => {
    fetchConsumerStatus();
  }, []);

  useEffect(() => {
    if (isMonitoring) {
      startMonitoring();
    } else {
      stopMonitoring();
    }
    return () => stopMonitoring();
  }, [isMonitoring]);

  useEffect(() => {
    scrollToBottom();
  }, [liveMessages]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  const fetchConsumerStatus = async () => {
    try {
      const response = await fetch('/api/consumer-status');
      const data = await response.json();
      
      if (data.success) {
        setConsumerStatus(data.consumers);
      }
    } catch (error) {
      console.error('Error fetching consumer status:', error);
    }
  };

  const fetchLiveMessages = async () => {
    try {
      const response = await fetch('/api/live-messages');
      const data = await response.json();
      
      if (data.success && data.messages.length > 0) {
        setLiveMessages(prev => {
          const newMessages = [...prev, ...data.messages];
          // Keep only the last maxMessages
          return newMessages.slice(-maxMessages);
        });
      }
    } catch (error) {
      console.error('Error fetching live messages:', error);
    }
  };

  const startMonitoring = () => {
    if (intervalRef.current) return;
    
    intervalRef.current = setInterval(() => {
      fetchLiveMessages();
    }, 1000); // Poll every second
  };

  const stopMonitoring = () => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  };

  const toggleMonitoring = () => {
    setIsMonitoring(!isMonitoring);
  };

  const clearMessages = () => {
    setLiveMessages([]);
  };

  const formatTimestamp = (timestamp) => {
    return new Date(timestamp).toLocaleString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      fractionalSecondDigits: 3
    });
  };

  const getDataTypeColor = (dataType) => {
    switch (dataType) {
      case 'edr': return '#3b82f6'; // Blue
      case 'ngav': return '#8b5cf6'; // Purple
      default: return '#6b7280'; // Gray
    }
  };

  const formatJsonPayload = (payload) => {
    try {
      const parsed = JSON.parse(payload);
      return JSON.stringify(parsed, null, 2);
    } catch {
      return payload;
    }
  };

  const renderConsumerStatus = () => {
    const consumers = Object.values(consumerStatus);
    
    if (consumers.length === 0) {
      return (
        <div className="status-container">
          <div className="status-item">
            <span className="status-label">状态:</span>
            <span className="status-value error">❌ 没有活跃的消费者</span>
          </div>
        </div>
      );
    }

    return (
      <div className="status-container">
        <div className="status-item">
          <span className="status-label">活跃消费者:</span>
          <span className="status-value">{consumers.length}</span>
        </div>
        <div className="consumers-list">
          {consumers.map((consumer, index) => (
            <div key={index} className="consumer-item">
              <div className="consumer-info">
                <span className="consumer-topic">{consumer.topic}</span>
                <span 
                  className="consumer-data-type"
                  style={{ backgroundColor: getDataTypeColor(consumer.data_type) }}
                >
                  {consumer.data_type?.toUpperCase() || 'UNKNOWN'}
                </span>
                <span className="consumer-status running">🟢 运行中</span>
              </div>
            </div>
          ))}
        </div>
      </div>
    );
  };

  const renderDataMonitoring = () => {
    return (
      <div className="data-monitoring">
        <div className="monitoring-header">
          <div className="monitoring-controls">
            <button 
              onClick={toggleMonitoring}
              className={`monitoring-button ${isMonitoring ? 'stop' : 'start'}`}
            >
              {isMonitoring ? '⏸️ 停止监听' : '▶️ 开始监听'}
            </button>
            <button onClick={clearMessages} className="clear-button">
              🗑️ 清空日志
            </button>
            <div className="max-messages-control">
              <label>最大消息数:</label>
              <input 
                type="number" 
                value={maxMessages} 
                onChange={(e) => setMaxMessages(parseInt(e.target.value) || 100)}
                min="10"
                max="1000"
                step="10"
              />
            </div>
          </div>
          
          {renderConsumerStatus()}
        </div>

        <div className="messages-container">
          <div className="messages-header">
            <h4>实时消息 ({liveMessages.length})</h4>
            {isMonitoring && <div className="monitoring-indicator">🔴 监听中</div>}
          </div>
          
          <div className="messages-list">
            {liveMessages.length === 0 ? (
              <div className="no-messages">
                {isMonitoring ? '等待消息...' : '点击"开始监听"以接收实时消息'}
              </div>
            ) : (
              liveMessages.map((message, index) => (
                <div key={index} className="message-item">
                  <div className="message-header">
                    <span className="message-time">
                      {formatTimestamp(message.timestamp || Date.now())}
                    </span>
                    <span className="message-topic">{message.topic}</span>
                    <span className="message-partition">
                      分区: {message.partition}, 偏移: {message.offset}
                    </span>
                    {message.data_type && (
                      <span 
                        className="message-data-type"
                        style={{ backgroundColor: getDataTypeColor(message.data_type) }}
                      >
                        {message.data_type.toUpperCase()}
                      </span>
                    )}
                  </div>
                  <div className="message-payload">
                    <pre>{formatJsonPayload(message.payload)}</pre>
                  </div>
                </div>
              ))
            )}
            <div ref={messagesEndRef} />
          </div>
        </div>
      </div>
    );
  };

  const renderSystemLogs = () => {
    return (
      <div className="system-logs">
        <h4>系统日志</h4>
        <p>系统日志功能待实现...</p>
      </div>
    );
  };

  return (
    <div className="logs-container">
      <div className="logs-header">
        <h2>日志管理</h2>
        <div className="logs-tabs">
          <button 
            className={`tab-button ${activeTab === 'monitoring' ? 'active' : ''}`}
            onClick={() => setActiveTab('monitoring')}
          >
            数据监听
          </button>
          <button 
            className={`tab-button ${activeTab === 'system' ? 'active' : ''}`}
            onClick={() => setActiveTab('system')}
          >
            系统日志
          </button>
        </div>
      </div>

      <div className="logs-content">
        {activeTab === 'monitoring' && renderDataMonitoring()}
        {activeTab === 'system' && renderSystemLogs()}
      </div>
    </div>
  );
};

export default Logs;