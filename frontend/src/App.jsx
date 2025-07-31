import React, { useState, useEffect } from 'react';
import axios from 'axios';
import './App.css';

function App() {
  const [activeView, setActiveView] = useState('home');
  const [configs, setConfigs] = useState([]);
  const [activeConfigs, setActiveConfigs] = useState([]);
  const [newConfig, setNewConfig] = useState({
    name: '',
    bootstrap_servers: '',
    topic: '',
    group_id: '',
    message_timeout_ms: 5000,
    request_timeout_ms: 5000,
    retry_backoff_ms: 100,
    retries: 3,
    auto_offset_reset: 'earliest',
    enable_auto_commit: true,
    auto_commit_interval_ms: 1000
  });
  const [editingConfig, setEditingConfig] = useState(null);
  const [showConfigForm, setShowConfigForm] = useState(false);
  const [saveResult, setSaveResult] = useState(null);
  const [tooltip, setTooltip] = useState({ show: false, content: '', x: 0, y: 0 });
  const [connectivityTests, setConnectivityTests] = useState({});
  const [testModal, setTestModal] = useState({ show: false, loading: false, result: null, error: null, config: null });

  useEffect(() => {
    loadConfigs();
    loadActiveConfigs();
  }, []);

  const loadConfigs = async () => {
    try {
      const response = await axios.get('/api/configs');
      if (response.data.success) {
        setConfigs(response.data.configs);
      }
    } catch (error) {
      console.error('Failed to load configs:', error);
    }
  };

  const loadActiveConfigs = async () => {
    try {
      const response = await axios.get('/api/configs/active');
      if (response.data.success) {
        setActiveConfigs(response.data.configs);
      }
    } catch (error) {
      console.error('Failed to load active configs:', error);
    }
  };

  const testConnectivity = async (config, showModal = false) => {
    const configId = config.id;
    
    if (showModal) {
      setTestModal({
        show: true,
        loading: true,
        result: null,
        error: null,
        config: config
      });
    } else {
      setConnectivityTests(prev => ({
        ...prev,
        [configId]: { loading: true, result: null, error: null }
      }));
    }
    
    try {
      const response = await axios.get('/api/test-connectivity', {
        params: {
          bootstrap_servers: config.bootstrap_servers,
          topic: config.topic
        }
      });
      
      if (showModal) {
        setTestModal(prev => ({
          ...prev,
          loading: false,
          result: response.data,
          error: null
        }));
      } else {
        setConnectivityTests(prev => ({
          ...prev,
          [configId]: {
            loading: false,
            result: response.data,
            error: null
          }
        }));
      }
      
      return response.data.success;
    } catch (error) {
      const errorMessage = error.response?.data?.message || error.message;
      
      if (showModal) {
        setTestModal(prev => ({
          ...prev,
          loading: false,
          result: null,
          error: errorMessage
        }));
      } else {
        setConnectivityTests(prev => ({
          ...prev,
          [configId]: {
            loading: false,
            result: null,
            error: errorMessage
          }
        }));
      }
      
      return false;
    }
  };

  const saveConfig = async () => {
    setSaveResult(null);
    
    try {
      let response;
      if (editingConfig) {
        response = await axios.put(`/api/config/${editingConfig.id}`, newConfig);
      } else {
        response = await axios.post('/api/config', newConfig);
      }
      
      setSaveResult(response.data);
      if (response.data.success) {
        setShowConfigForm(false);
        setEditingConfig(null);
        loadConfigs();
        loadActiveConfigs();
        resetConfigForm();
      }
    } catch (error) {
      setSaveResult({
        success: false,
        message: error.response?.data?.message || error.message
      });
    }
  };

  const resetConfigForm = () => {
    setNewConfig({
      name: '',
      bootstrap_servers: '',
      topic: '',
      group_id: '',
      message_timeout_ms: 5000,
      request_timeout_ms: 5000,
      retry_backoff_ms: 100,
      retries: 3,
      auto_offset_reset: 'earliest',
      enable_auto_commit: true,
      auto_commit_interval_ms: 1000
    });
  };

  const startEditConfig = (config) => {
    setEditingConfig(config);
    setNewConfig({
      name: config.name,
      bootstrap_servers: config.bootstrap_servers,
      topic: config.topic,
      group_id: config.group_id,
      message_timeout_ms: config.message_timeout_ms,
      request_timeout_ms: config.request_timeout_ms,
      retry_backoff_ms: config.retry_backoff_ms,
      retries: config.retries,
      auto_offset_reset: config.auto_offset_reset,
      enable_auto_commit: config.enable_auto_commit,
      auto_commit_interval_ms: config.auto_commit_interval_ms,
    });
    setShowConfigForm(true);
  };

  const deleteConfig = async (configId) => {
    if (!window.confirm('确定要删除这个配置吗？')) {
      return;
    }

    try {
      const response = await axios.delete(`/api/config/${configId}`);
      if (response.data.success) {
        loadConfigs();
        loadActiveConfigs();
      }
    } catch (error) {
      console.error('Failed to delete config:', error);
    }
  };

  const toggleConfigActive = async (configId, isActive) => {
    if (isActive) {
      // 激活配置前先测试连接
      const config = configs.find(c => c.id === configId);
      if (config) {
        const connectionSuccess = await testConnectivity(config);
        if (!connectionSuccess) {
          alert('连接测试失败，无法激活配置！');
          return;
        }
      }
    }

    try {
      const response = await axios.post(`/api/config/${configId}/toggle`, { is_active: isActive });
      if (response.data.success) {
        loadConfigs();
        loadActiveConfigs();
      }
    } catch (error) {
      console.error('Failed to toggle config:', error);
    }
  };

  const closeTestModal = () => {
    setTestModal({ show: false, loading: false, result: null, error: null, config: null });
  };

  const showTooltip = (event, config) => {
    const rect = event.target.getBoundingClientRect();
    setTooltip({
      show: true,
      content: config,
      x: rect.left + rect.width / 2,
      y: rect.top - 10
    });
  };

  const hideTooltip = () => {
    setTooltip({ show: false, content: '', x: 0, y: 0 });
  };

  const cancelEdit = () => {
    setShowConfigForm(false);
    setEditingConfig(null);
    setSaveResult(null);
    resetConfigForm();
  };

  const renderHome = () => (
    <div className="home-content">
      <div className="welcome-section">
        <h2>欢迎使用挖掘告警系统</h2>
        <p>这是一个基于Kafka的安全告警处理系统，支持多种数据源的告警收集和处理。</p>
      </div>
      
      <div className="status-overview">
        <div className="status-card">
          <h3>当前活跃配置</h3>
          <div className="status-number">{activeConfigs.length}</div>
          <p>个Kafka配置正在运行</p>
        </div>
        
        <div className="status-card">
          <h3>总配置数量</h3>
          <div className="status-number">{configs.length}</div>
          <p>个Kafka配置已创建</p>
        </div>
      </div>

      {activeConfigs.length > 0 && (
        <div className="active-configs-section">
          <h3>🟢 当前活跃的Kafka配置</h3>
          <div className="active-configs-list">
            {activeConfigs.map((cfg) => (
              <div key={cfg.id} className="active-config-badge">
                <span 
                  className="config-name"
                  onMouseEnter={(e) => showTooltip(e, cfg)}
                  onMouseLeave={hideTooltip}
                >
                  {cfg.name}
                </span>
                <span className="config-server">{cfg.bootstrap_servers}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );

  const renderKafkaConfig = () => (
    <div className="kafka-config-content">
      <div className="config-header">
        <h2>Kafka配置管理</h2>
        <div className="config-actions">
          <button 
            onClick={() => setShowConfigForm(!showConfigForm)}
            className="btn btn-primary"
          >
            {showConfigForm ? '取消' : '新增配置'}
          </button>
          {showConfigForm && editingConfig && (
            <button 
              onClick={cancelEdit}
              className="btn btn-secondary"
            >
              取消编辑
            </button>
          )}
        </div>
      </div>

      {showConfigForm && (
        <div className="config-form">
          <h3>{editingConfig ? '编辑配置' : '创建新配置'}</h3>
          <div className="form-grid">
            <div className="form-row">
              <label>配置名称：</label>
              <input
                type="text"
                value={newConfig.name}
                onChange={(e) => setNewConfig({...newConfig, name: e.target.value})}
                className="input"
                placeholder="输入配置名称"
              />
            </div>
            
            <div className="form-row">
              <label>服务器地址：</label>
              <input
                type="text"
                value={newConfig.bootstrap_servers}
                onChange={(e) => setNewConfig({...newConfig, bootstrap_servers: e.target.value})}
                className="input"
                placeholder="例如：localhost:9092"
              />
            </div>
            
            <div className="form-row">
              <label>主题名称：</label>
              <input
                type="text"
                value={newConfig.topic}
                onChange={(e) => setNewConfig({...newConfig, topic: e.target.value})}
                className="input"
                placeholder="例如：alerts"
              />
            </div>
            
            <div className="form-row">
              <label>消费者组ID：</label>
              <input
                type="text"
                value={newConfig.group_id}
                onChange={(e) => setNewConfig({...newConfig, group_id: e.target.value})}
                className="input"
                placeholder="例如：alerts-consumer-group"
              />
            </div>

            <div className="form-row">
              <label>消息超时时间（毫秒）：</label>
              <input
                type="number"
                value={newConfig.message_timeout_ms}
                onChange={(e) => setNewConfig({...newConfig, message_timeout_ms: parseInt(e.target.value)})}
                className="input"
              />
            </div>

            <div className="form-row">
              <label>请求超时时间（毫秒）：</label>
              <input
                type="number"
                value={newConfig.request_timeout_ms}
                onChange={(e) => setNewConfig({...newConfig, request_timeout_ms: parseInt(e.target.value)})}
                className="input"
              />
            </div>
          </div>
          
          <button onClick={saveConfig} className="btn btn-primary">
            {editingConfig ? '更新配置' : '保存配置'}
          </button>

          {saveResult && (
            <div className={`result ${saveResult.success ? 'success' : 'error'}`}>
              <p>{saveResult.message}</p>
            </div>
          )}
        </div>
      )}

      {/* 配置列表 */}
      {configs.length > 0 && (
        <div className="configs-list">
          <h3>已保存的配置</h3>
          <div className="configs-grid">
            {configs.map((cfg) => (
              <div key={cfg.id} className={`config-card ${cfg.is_active ? 'active' : ''}`}>
                <div className="config-header">
                  <h4>{cfg.name}</h4>
                  {cfg.is_active && <span className="active-badge">活跃</span>}
                </div>
                <div className="config-details">
                  <p><strong>服务器：</strong> {cfg.bootstrap_servers}</p>
                  <p><strong>主题：</strong> {cfg.topic}</p>
                  <p><strong>消费者组：</strong> {cfg.group_id}</p>
                  <p><strong>创建时间：</strong> {new Date(cfg.created_at).toLocaleDateString()}</p>
                </div>

                {/* 连接测试结果 */}
                {connectivityTests[cfg.id] && (
                  <div className="connectivity-result">
                    {connectivityTests[cfg.id].loading && (
                      <div className="loading">正在测试连接...</div>
                    )}
                    {connectivityTests[cfg.id].result && (
                      <div className={`result ${connectivityTests[cfg.id].result.success ? 'success' : 'error'}`}>
                        <small>
                          {connectivityTests[cfg.id].result.success ? '✅ 连接成功' : '❌ 连接失败'}
                        </small>
                      </div>
                    )}
                    {connectivityTests[cfg.id].error && (
                      <div className="result error">
                        <small>❌ {connectivityTests[cfg.id].error}</small>
                      </div>
                    )}
                  </div>
                )}

                <div className="config-actions">
                  <button 
                    onClick={() => testConnectivity(cfg, true)}
                    className="btn btn-info btn-sm"
                    disabled={connectivityTests[cfg.id]?.loading}
                  >
                    测试连接
                  </button>
                  
                  <button 
                    onClick={() => toggleConfigActive(cfg.id, !cfg.is_active)}
                    className={`btn btn-sm ${cfg.is_active ? 'btn-warning' : 'btn-success'}`}
                  >
                    {cfg.is_active ? '停用' : '启用'}
                  </button>
                  
                  <button 
                    onClick={() => startEditConfig(cfg)}
                    className="btn btn-secondary btn-sm"
                  >
                    编辑
                  </button>
                  
                  <button 
                    onClick={() => deleteConfig(cfg.id)}
                    className="btn btn-danger btn-sm"
                  >
                    删除
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );

  return (
    <div className="app">
      <header className="app-header">
        {/* <h1>🔍 挖掘告警系统</h1> */}
        <h1>挖掘告警系统</h1>

        {/* <p>基于Kafka的安全告警处理平台</p> */}
      </header>

      <div className="app-container">
        {/* 侧边栏 */}
        <nav className="sidebar">
          <ul className="nav-menu">
            <li className={activeView === 'home' ? 'active' : ''}>
              <button onClick={() => setActiveView('home')}>
                🏠 首页
              </button>
            </li>
            <li className={activeView === 'kafka' ? 'active' : ''}>
              <button onClick={() => setActiveView('kafka')}>
                ⚙️ Kafka配置
              </button>
            </li>
          </ul>
        </nav>

        {/* 主内容区域 */}
        <main className="main-content">
          {activeView === 'home' && renderHome()}
          {activeView === 'kafka' && renderKafkaConfig()}
        </main>
      </div>

      {/* 工具提示 */}
      {tooltip.show && (
        <div 
          className="tooltip"
          style={{
            left: tooltip.x,
            top: tooltip.y,
            transform: 'translateX(-50%) translateY(-100%)'
          }}
        >
          <div className="tooltip-content">
            <h4>{tooltip.content.name}</h4>
            <p><strong>服务器：</strong> {tooltip.content.bootstrap_servers}</p>
            <p><strong>主题：</strong> {tooltip.content.topic}</p>
            <p><strong>消费者组：</strong> {tooltip.content.group_id}</p>
            <p><strong>超时时间：</strong> {tooltip.content.message_timeout_ms}ms</p>
            <p><strong>重试次数：</strong> {tooltip.content.retries}</p>
            <p><strong>偏移重置：</strong> {tooltip.content.auto_offset_reset}</p>
            <p><strong>自动提交：</strong> {tooltip.content.enable_auto_commit ? '是' : '否'}</p>
          </div>
        </div>
      )}

      {/* 连接测试弹框 */}
      {testModal.show && (
        <div className="modal-overlay" onClick={closeTestModal}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h3>🔗 连接测试结果</h3>
              <button className="modal-close" onClick={closeTestModal}>
                ×
              </button>
            </div>
            
            {testModal.config && (
              <div className="modal-body">
                <div className="test-config-info">
                  <h4>📋 测试配置信息</h4>
                  <div className="config-info-grid">
                    <div><strong>配置名称：</strong>{testModal.config.name}</div>
                    <div><strong>服务器地址：</strong>{testModal.config.bootstrap_servers}</div>
                    <div><strong>主题名称：</strong>{testModal.config.topic}</div>
                    <div><strong>消费者组：</strong>{testModal.config.group_id}</div>
                  </div>
                </div>
                
                <div className="test-result-section">
                  <h4>📊 测试结果</h4>
                  
                  {testModal.loading && (
                    <div className="test-loading">
                      <div className="loading-spinner"></div>
                      <span>正在连接Kafka服务器，请稍候...</span>
                    </div>
                  )}
                  
                  {testModal.result && (
                    <div className={`test-result ${testModal.result.success ? 'success' : 'error'}`}>
                      <div className="result-status">
                        {testModal.result.success ? (
                          <><span className="status-icon">✅</span> 连接成功！</>
                        ) : (
                          <><span className="status-icon">❌</span> 连接失败！</>
                        )}
                      </div>
                      
                      <div className="result-message">
                        <strong>详细信息：</strong>
                        <p>{testModal.result.message}</p>
                      </div>
                      
                      {testModal.result.details && (
                        <div className="result-details">
                          <strong>连接详情：</strong>
                          <pre>{JSON.stringify(testModal.result.details, null, 2)}</pre>
                        </div>
                      )}
                    </div>
                  )}
                  
                  {testModal.error && (
                    <div className="test-result error">
                      <div className="result-status">
                        <span className="status-icon">❌</span> 连接失败！
                      </div>
                      
                      <div className="result-message">
                        <strong>错误信息：</strong>
                        <p>{testModal.error}</p>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            )}
            
            <div className="modal-footer">
              <button className="btn btn-secondary" onClick={closeTestModal}>
                关闭
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;