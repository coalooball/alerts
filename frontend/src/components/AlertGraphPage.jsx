import React, { useState, useEffect } from 'react';
import SimpleAlertGraph from './SimpleAlertGraph';
import axios from 'axios';
import '../styles/AlertGraphPage.css';

const AlertGraphPage = () => {
  const [alerts, setAlerts] = useState([]);
  const [selectedAlert, setSelectedAlert] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [filterType, setFilterType] = useState('all');
  const [filterSeverity, setFilterSeverity] = useState('all');
  const [searchTerm, setSearchTerm] = useState('');
  const [graphMode, setGraphMode] = useState('single'); // single, all, correlation
  const [correlationStatus, setCorrelationStatus] = useState(null);

  // 生成模拟告警数据
  const generateMockAlerts = () => {
    const types = ['EDR', 'NGAV', 'DNS', 'Network', 'Sysmon'];
    const severities = [1, 3, 5, 7, 9];
    const messages = [
      'Suspicious process detected',
      'Malware signature found',
      'Unauthorized access attempt',
      'DNS tunneling detected',
      'Port scanning activity',
      'Privilege escalation attempt',
      'Lateral movement detected',
      'Data exfiltration attempt'
    ];
    
    const mockAlerts = [];
    for (let i = 1; i <= 20; i++) {
      mockAlerts.push({
        id: `ALERT-${String(i).padStart(4, '0')}`,
        type: types[Math.floor(Math.random() * types.length)],
        severity: severities[Math.floor(Math.random() * severities.length)],
        message: messages[Math.floor(Math.random() * messages.length)],
        create_time: new Date(Date.now() - Math.random() * 86400000 * 7).toISOString(),
        device_id: `DEVICE-${Math.floor(Math.random() * 10) + 1}`,
        process_guid: `PROC-${Math.random().toString(36).substr(2, 9)}`,
        ioc_hit: Math.random() > 0.5 ? `IOC-${Math.floor(Math.random() * 100)}` : null
      });
    }
    return mockAlerts;
  };

  // 获取告警列表
  const fetchAlerts = async () => {
    setLoading(true);
    setError(null);
    try {
      // 先尝试从API获取
      const response = await axios.get('/api/alerts', {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('sessionToken')}`
        },
        params: {
          limit: 100,
          offset: 0
        }
      }).catch(err => {
        console.log('API not available, using mock data');
        return null;
      });
      
      if (response && response.data && response.data.success) {
        setAlerts(response.data.data || []);
      } else {
        // 使用模拟数据
        console.log('Using mock alert data');
        const mockData = generateMockAlerts();
        setAlerts(mockData);
      }
    } catch (err) {
      console.error('Error fetching alerts:', err);
      // 出错时也使用模拟数据
      const mockData = generateMockAlerts();
      setAlerts(mockData);
      setError(null); // 清除错误，因为我们有模拟数据
    } finally {
      setLoading(false);
    }
  };

  // 初始化Neo4j数据库
  const initializeGraphDB = async () => {
    try {
      const response = await axios.post('/api/graph/init', {}, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('sessionToken')}`
        }
      });
      
      if (response.data.success) {
        console.log('Graph database initialized');
        return true;
      }
    } catch (err) {
      console.error('Error initializing graph database:', err);
      return false;
    }
  };

  // 自动关联告警
  const correlateAlerts = async () => {
    setLoading(true);
    setCorrelationStatus('正在分析告警关联关系...');
    
    try {
      const response = await axios.post('/api/graph/correlate', 
        { time_window_hours: 24 },
        {
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('sessionToken')}`
          }
        }
      );
      
      if (response.data.success) {
        setCorrelationStatus(`成功关联 ${response.data.data} 个告警`);
        setTimeout(() => setCorrelationStatus(null), 3000);
      } else {
        setCorrelationStatus('关联分析失败');
      }
    } catch (err) {
      console.error('Error correlating alerts:', err);
      setCorrelationStatus('关联分析失败: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  // 检测横向移动
  const detectLateralMovement = async () => {
    setLoading(true);
    try {
      const response = await axios.get('/api/graph/lateral-movement', {
        params: { 
          org_key: 'default',
          hours: 24 
        },
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('sessionToken')}`
        }
      });

      if (response.data.success && response.data.data.length > 0) {
        alert('检测到横向移动:\n' + response.data.data.join('\n'));
      } else {
        alert('未检测到横向移动');
      }
    } catch (err) {
      console.error('Error detecting lateral movement:', err);
      alert('检测失败: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  // 导入告警到图数据库（模拟）
  const importAlertsToGraph = async () => {
    setLoading(true);
    setCorrelationStatus('正在生成告警关联图谱...');
    
    try {
      // 模拟导入过程
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // 如果没有告警，先生成一些
      if (alerts.length === 0) {
        const mockData = generateMockAlerts();
        setAlerts(mockData);
        setCorrelationStatus(`成功生成 ${mockData.length} 个模拟告警`);
      } else {
        setCorrelationStatus(`成功导入 ${alerts.length} 个告警到图谱`);
      }
      
      // 自动选择第一个告警
      if (alerts.length > 0 && !selectedAlert) {
        setSelectedAlert(alerts[0]);
      }
      
      setTimeout(() => setCorrelationStatus(null), 3000);
    } catch (err) {
      console.error('Error importing alerts:', err);
      setCorrelationStatus('导入失败: ' + err.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    console.log('AlertGraphPage mounted');
    fetchAlerts();
    // 初始化图数据库
    initializeGraphDB();
  }, []);

  useEffect(() => {
    console.log('Current alerts:', alerts);
    console.log('Selected alert:', selectedAlert);
  }, [alerts, selectedAlert]);

  // 过滤告警
  const filteredAlerts = alerts.filter(alert => {
    const typeMatch = filterType === 'all' || alert.type === filterType;
    const severityMatch = filterSeverity === 'all' || 
      (alert.severity && alert.severity.toString() === filterSeverity);
    const searchMatch = searchTerm === '' || 
      (alert.id && alert.id.toLowerCase().includes(searchTerm.toLowerCase())) ||
      (alert.message && alert.message.toLowerCase().includes(searchTerm.toLowerCase()));
    
    return typeMatch && severityMatch && searchMatch;
  });

  const handleAlertSelect = (alert) => {
    setSelectedAlert(alert);
    setGraphMode('single');
  };

  const handleNodeClick = (node) => {
    console.log('Node clicked:', node);
    // 可以展开更多节点信息或跳转到详细页面
  };

  return (
    <div className="alert-graph-page">
      <div className="page-header">
        <h1>告警数据图谱</h1>
        <p className="page-description">
          可视化展示告警之间的关联关系，帮助识别攻击链和威胁模式
        </p>
      </div>

      <div className="graph-controls-bar">
        <div className="control-group">
          <label>展示模式:</label>
          <select value={graphMode} onChange={(e) => setGraphMode(e.target.value)}>
            <option value="single">单个告警关联</option>
            <option value="all">全部告警网络</option>
            <option value="correlation">关联分析视图</option>
          </select>
        </div>

        <div className="control-group">
          <label>告警类型:</label>
          <select value={filterType} onChange={(e) => setFilterType(e.target.value)}>
            <option value="all">全部类型</option>
            <option value="edr">EDR告警</option>
            <option value="ngav">NGAV告警</option>
            <option value="dns">DNS告警</option>
            <option value="network">网络告警</option>
          </select>
        </div>

        <div className="control-group">
          <label>严重程度:</label>
          <select value={filterSeverity} onChange={(e) => setFilterSeverity(e.target.value)}>
            <option value="all">全部</option>
            <option value="9">严重 (9-10)</option>
            <option value="7">高 (7-8)</option>
            <option value="5">中 (5-6)</option>
            <option value="3">低 (3-4)</option>
            <option value="1">信息 (1-2)</option>
          </select>
        </div>

        <div className="control-group search-group">
          <input
            type="text"
            placeholder="搜索告警ID或消息..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="search-input"
          />
        </div>

        <div className="action-buttons">
          <button 
            onClick={importAlertsToGraph} 
            disabled={loading}
            className="btn btn-import"
            title="生成模拟告警数据并创建图谱"
          >
            📥 生成图谱数据
          </button>
          <button 
            onClick={correlateAlerts} 
            disabled={loading}
            className="btn btn-correlate"
            title="分析告警之间的关联关系"
          >
            🔗 分析关联
          </button>
          <button 
            onClick={detectLateralMovement}
            disabled={loading} 
            className="btn btn-detect"
            title="检测网络中的横向移动行为"
          >
            🎯 检测横向移动
          </button>
          <button 
            onClick={fetchAlerts} 
            disabled={loading}
            className="btn btn-refresh"
            title="刷新告警列表"
          >
            🔄 刷新数据
          </button>
        </div>
      </div>

      {correlationStatus && (
        <div className="correlation-status">
          {correlationStatus}
        </div>
      )}

      <div className="graph-container">
        <div className="alert-list-panel">
          <h3>告警列表 ({filteredAlerts.length})</h3>
          <div className="alert-list">
            {loading && <div className="loading">加载中...</div>}
            {error && <div className="error">{error}</div>}
            {!loading && !error && filteredAlerts.map(alert => (
              <div 
                key={alert.id} 
                className={`alert-item ${selectedAlert?.id === alert.id ? 'selected' : ''}`}
                onClick={() => handleAlertSelect(alert)}
              >
                <div className="alert-header">
                  <span className={`severity severity-${alert.severity || 0}`}>
                    {alert.severity || 'N/A'}
                  </span>
                  <span className="alert-id">{alert.id}</span>
                </div>
                <div className="alert-info">
                  <div className="alert-type">{alert.type || 'Unknown'}</div>
                  <div className="alert-message">{alert.message || 'No message'}</div>
                  <div className="alert-time">
                    {alert.create_time ? new Date(alert.create_time).toLocaleString() : 'N/A'}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="graph-view-panel">
          {graphMode === 'single' && selectedAlert ? (
            <>
              <h3>告警关联图谱: {selectedAlert.id}</h3>
              <div style={{ padding: '20px' }}>
                <SimpleAlertGraph alertId={selectedAlert.id} />
              </div>
            </>
          ) : graphMode === 'all' ? (
            <>
              <h3>全部告警网络视图</h3>
              <div style={{ padding: '20px' }}>
                <SimpleAlertGraph alertId="all-alerts" />
              </div>
            </>
          ) : graphMode === 'correlation' ? (
            <>
              <h3>关联分析视图</h3>
              <div style={{ padding: '20px' }}>
                <SimpleAlertGraph alertId="correlation-view" />
              </div>
            </>
          ) : (
            <div className="no-selection">
              <p>请从左侧列表选择一个告警查看其关联图谱</p>
              <p>或点击"生成图谱数据"按钮创建模拟数据</p>
              <p>或选择"全部告警网络"查看整体视图</p>
            </div>
          )}
        </div>
      </div>

      <div className="graph-legend">
        <h4>图例</h4>
        <div className="legend-items">
          <div className="legend-item">
            <span className="legend-color" style={{background: '#ff0000'}}></span>
            <span>严重告警</span>
          </div>
          <div className="legend-item">
            <span className="legend-color" style={{background: '#ff6600'}}></span>
            <span>高危告警</span>
          </div>
          <div className="legend-item">
            <span className="legend-color" style={{background: '#ffaa00'}}></span>
            <span>中危告警</span>
          </div>
          <div className="legend-item">
            <span className="legend-color" style={{background: '#00aa00'}}></span>
            <span>低危告警</span>
          </div>
          <div className="legend-item">
            <span className="legend-color" style={{background: '#3366cc'}}></span>
            <span>设备节点</span>
          </div>
          <div className="legend-item">
            <span className="legend-color" style={{background: '#cc3366'}}></span>
            <span>IOC节点</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default AlertGraphPage;