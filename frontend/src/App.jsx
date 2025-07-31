import React, { useState, useEffect } from 'react';
import axios from 'axios';
import './App.css';

function App() {
  const [config, setConfig] = useState({});
  const [configs, setConfigs] = useState([]);
  const [activeConfigs, setActiveConfigs] = useState([]);
  const [connectivityTest, setConnectivityTest] = useState({
    loading: false,
    result: null,
    error: null
  });
  const [customConfig, setCustomConfig] = useState({
    bootstrap_servers: '',
    topic: ''
  });
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

  useEffect(() => {
    loadConfig();
    loadConfigs();
    loadActiveConfigs();
  }, []);

  const loadConfig = async () => {
    try {
      const response = await axios.get('/api/config');
      setConfig(response.data);
      setCustomConfig({
        bootstrap_servers: response.data.bootstrap_servers || '',
        topic: response.data.topic || ''
      });
    } catch (error) {
      console.error('Failed to load config:', error);
    }
  };

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

  const testConnectivity = async (useCustom = false) => {
    setConnectivityTest({ loading: true, result: null, error: null });
    
    try {
      const params = useCustom ? {
        bootstrap_servers: customConfig.bootstrap_servers,
        topic: customConfig.topic
      } : {};
      
      const response = await axios.get('/api/test-connectivity', { params });
      setConnectivityTest({
        loading: false,
        result: response.data,
        error: null
      });
    } catch (error) {
      setConnectivityTest({
        loading: false,
        result: null,
        error: error.response?.data?.message || error.message
      });
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
    if (!window.confirm('Are you sure you want to delete this configuration?')) {
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
    try {
      const response = await axios.post(`/api/config/${configId}/toggle`, { is_active: isActive });
      if (response.data.success) {
        loadConfigs();
        loadActiveConfigs();
        loadConfig(); // Reload main config if it changed
      }
    } catch (error) {
      console.error('Failed to toggle config:', error);
    }
  };

  const activateConfig = async (configId) => {
    try {
      const response = await axios.post(`/api/config/${configId}/activate`);
      if (response.data.success) {
        loadConfig();
        loadConfigs();
        loadActiveConfigs();
      }
    } catch (error) {
      console.error('Failed to activate config:', error);
    }
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

  return (
    <div className="app">
      <header className="app-header">
        <h1>🚨 Kafka Configuration Dashboard</h1>
        <p>Manage Kafka configurations and test connectivity</p>
      </header>

      <main className="main-content">
        {/* Active Configurations Section */}
        <section className="card">
          <h2>📡 Active Kafka Configurations ({activeConfigs.length})</h2>
          {activeConfigs.length === 0 ? (
            <div className="no-configs">
              <p>No active configurations. Please activate at least one configuration to use Kafka.</p>
            </div>
          ) : (
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
                  <button 
                    onClick={() => toggleConfigActive(cfg.id, false)}
                    className="deactivate-btn"
                    title="Deactivate this configuration"
                  >
                    ×
                  </button>
                </div>
              ))}
            </div>
          )}
        </section>

        {/* Connectivity Test Section */}
        <section className="card">
          <h2>🔌 Connectivity Test</h2>
          
          <div className="connectivity-controls">
            <button 
              onClick={() => testConnectivity(false)}
              disabled={connectivityTest.loading}
              className="btn btn-primary"
            >
              {connectivityTest.loading ? 'Testing...' : 'Test Active Config'}
            </button>
            
            <div className="custom-config">
              <h3>Test Custom Configuration</h3>
              <input
                type="text"
                placeholder="Bootstrap Servers (e.g., localhost:9092)"
                value={customConfig.bootstrap_servers}
                onChange={(e) => setCustomConfig({
                  ...customConfig,
                  bootstrap_servers: e.target.value
                })}
                className="input"
              />
              <input
                type="text"
                placeholder="Topic"
                value={customConfig.topic}
                onChange={(e) => setCustomConfig({
                  ...customConfig,
                  topic: e.target.value
                })}
                className="input"
              />
              <button 
                onClick={() => testConnectivity(true)}
                disabled={connectivityTest.loading}
                className="btn btn-secondary"
              >
                Test Custom Config
              </button>
            </div>
          </div>

          {connectivityTest.result && (
            <div className={`result ${connectivityTest.result.success ? 'success' : 'error'}`}>
              <h3>{connectivityTest.result.success ? '✅ Success' : '❌ Failed'}</h3>
              <p>{connectivityTest.result.message}</p>
              {connectivityTest.result.details && (
                <pre>{JSON.stringify(connectivityTest.result.details, null, 2)}</pre>
              )}
            </div>
          )}

          {connectivityTest.error && (
            <div className="result error">
              <h3>❌ Error</h3>
              <p>{connectivityTest.error}</p>
            </div>
          )}
        </section>

        {/* Configurations Management */}
        <section className="card">
          <h2>⚙️ Configuration Management</h2>
          
          <div className="config-actions">
            <button 
              onClick={() => setShowConfigForm(!showConfigForm)}
              className="btn btn-primary"
            >
              {showConfigForm ? 'Cancel' : 'Add New Configuration'}
            </button>
            {showConfigForm && editingConfig && (
              <button 
                onClick={cancelEdit}
                className="btn btn-secondary"
              >
                Cancel Edit
              </button>
            )}
          </div>

          {showConfigForm && (
            <div className="config-form">
              <h3>{editingConfig ? 'Edit Configuration' : 'Create New Configuration'}</h3>
              <div className="form-grid">
                <div className="form-row">
                  <label>Name:</label>
                  <input
                    type="text"
                    value={newConfig.name}
                    onChange={(e) => setNewConfig({...newConfig, name: e.target.value})}
                    className="input"
                    placeholder="Configuration name"
                  />
                </div>
                
                <div className="form-row">
                  <label>Bootstrap Servers:</label>
                  <input
                    type="text"
                    value={newConfig.bootstrap_servers}
                    onChange={(e) => setNewConfig({...newConfig, bootstrap_servers: e.target.value})}
                    className="input"
                    placeholder="localhost:9092"
                  />
                </div>
                
                <div className="form-row">
                  <label>Topic:</label>
                  <input
                    type="text"
                    value={newConfig.topic}
                    onChange={(e) => setNewConfig({...newConfig, topic: e.target.value})}
                    className="input"
                    placeholder="alerts"
                  />
                </div>
                
                <div className="form-row">
                  <label>Group ID:</label>
                  <input
                    type="text"
                    value={newConfig.group_id}
                    onChange={(e) => setNewConfig({...newConfig, group_id: e.target.value})}
                    className="input"
                    placeholder="alerts-consumer-group"
                  />
                </div>
              </div>
              
              <button onClick={saveConfig} className="btn btn-primary">
                {editingConfig ? 'Update Configuration' : 'Save Configuration'}
              </button>

              {saveResult && (
                <div className={`result ${saveResult.success ? 'success' : 'error'}`}>
                  <p>{saveResult.message}</p>
                </div>
              )}
            </div>
          )}

          {/* Existing Configurations List */}
          {configs.length > 0 && (
            <div className="configs-list">
              <h3>📋 Saved Configurations</h3>
              <div className="configs-grid">
                {configs.map((cfg) => (
                  <div key={cfg.id} className={`config-card ${cfg.is_active ? 'active' : ''}`}>
                    <div className="config-header">
                      <h4>{cfg.name}</h4>
                      {cfg.is_active && <span className="active-badge">Active</span>}
                    </div>
                    <div className="config-details">
                      <p><strong>Server:</strong> {cfg.bootstrap_servers}</p>
                      <p><strong>Topic:</strong> {cfg.topic}</p>
                      <p><strong>Group:</strong> {cfg.group_id}</p>
                      <p><strong>Created:</strong> {new Date(cfg.created_at).toLocaleDateString()}</p>
                    </div>
                    <div className="config-actions">
                      <button 
                        onClick={() => toggleConfigActive(cfg.id, !cfg.is_active)}
                        className={`btn btn-sm ${cfg.is_active ? 'btn-warning' : 'btn-success'}`}
                      >
                        {cfg.is_active ? 'Deactivate' : 'Activate'}
                      </button>
                      <button 
                        onClick={() => startEditConfig(cfg)}
                        className="btn btn-secondary btn-sm"
                      >
                        Edit
                      </button>
                      <button 
                        onClick={() => deleteConfig(cfg.id)}
                        className="btn btn-danger btn-sm"
                      >
                        Delete
                      </button>
                      {!cfg.is_active && (
                        <button 
                          onClick={() => activateConfig(cfg.id)}
                          className="btn btn-primary btn-sm"
                          title="Activate only this configuration (deactivate others)"
                        >
                          Set as Only Active
                        </button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </section>
      </main>

      {/* Tooltip */}
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
            <p><strong>Server:</strong> {tooltip.content.bootstrap_servers}</p>
            <p><strong>Topic:</strong> {tooltip.content.topic}</p>
            <p><strong>Group:</strong> {tooltip.content.group_id}</p>
            <p><strong>Timeout:</strong> {tooltip.content.message_timeout_ms}ms</p>
            <p><strong>Retries:</strong> {tooltip.content.retries}</p>
            <p><strong>Offset Reset:</strong> {tooltip.content.auto_offset_reset}</p>
            <p><strong>Auto Commit:</strong> {tooltip.content.enable_auto_commit ? 'Yes' : 'No'}</p>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;