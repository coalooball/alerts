import React, { useState, useEffect } from 'react';
import axios from 'axios';
import './App.css';

function App() {
  const [config, setConfig] = useState({});
  const [configs, setConfigs] = useState([]);
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
  const [showConfigForm, setShowConfigForm] = useState(false);
  const [saveResult, setSaveResult] = useState(null);

  useEffect(() => {
    loadConfig();
    loadConfigs();
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
      const response = await axios.post('/api/config', newConfig);
      setSaveResult(response.data);
      if (response.data.success) {
        setShowConfigForm(false);
        loadConfigs();
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
      }
    } catch (error) {
      setSaveResult({
        success: false,
        message: error.response?.data?.message || error.message
      });
    }
  };

  const activateConfig = async (configId) => {
    try {
      const response = await axios.post(`/api/config/${configId}/activate`);
      if (response.data.success) {
        loadConfig();
        loadConfigs();
      }
    } catch (error) {
      console.error('Failed to activate config:', error);
    }
  };

  return (
    <div className="app">
      <header className="app-header">
        <h1>🚨 Kafka Configuration Dashboard</h1>
        <p>Manage Kafka configurations and test connectivity</p>
      </header>

      <main className="main-content">
        {/* Current Configuration Section */}
        <section className="card">
          <h2>📡 Current Active Configuration</h2>
          <div className="config-display">
            <div className="config-item">
              <strong>Bootstrap Servers:</strong> {config.bootstrap_servers}
            </div>
            <div className="config-item">
              <strong>Topic:</strong> {config.topic}
            </div>
            <div className="config-item">
              <strong>Group ID:</strong> {config.group_id}
            </div>
          </div>
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
          
          <button 
            onClick={() => setShowConfigForm(!showConfigForm)}
            className="btn btn-primary"
          >
            {showConfigForm ? 'Cancel' : 'Add New Configuration'}
          </button>

          {showConfigForm && (
            <div className="config-form">
              <h3>Create New Configuration</h3>
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
                Save Configuration
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
                    {!cfg.is_active && (
                      <button 
                        onClick={() => activateConfig(cfg.id)}
                        className="btn btn-secondary btn-sm"
                      >
                        Activate
                      </button>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}
        </section>
      </main>
    </div>
  );
}

export default App;