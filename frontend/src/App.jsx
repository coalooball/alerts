import React, { useState, useEffect } from 'react';
import axios from 'axios';
import './App.css';

function App() {
  const [config, setConfig] = useState({});
  const [connectivityTest, setConnectivityTest] = useState({
    loading: false,
    result: null,
    error: null
  });
  const [customConfig, setCustomConfig] = useState({
    bootstrap_servers: '',
    topic: ''
  });
  const [messageForm, setMessageForm] = useState({
    type: 'alert',
    data: '',
    topic: ''
  });
  const [sendResult, setSendResult] = useState(null);
  const [consumeResult, setConsumeResult] = useState({
    loading: false,
    messages: [],
    error: null
  });

  useEffect(() => {
    loadConfig();
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

  const sendMessage = async () => {
    setSendResult(null);
    
    try {
      let data;
      try {
        data = JSON.parse(messageForm.data);
      } catch {
        data = messageForm.data;
      }
      
      const response = await axios.post('/api/send-message', {
        message_type: messageForm.type,
        data: data,
        topic: messageForm.topic || undefined
      });
      
      setSendResult(response.data);
    } catch (error) {
      setSendResult({
        success: false,
        message: error.response?.data?.message || error.message
      });
    }
  };

  const consumeMessages = async () => {
    setConsumeResult({ loading: true, messages: [], error: null });
    
    try {
      const response = await axios.get('/api/consume-messages', {
        params: {
          max_messages: 10,
          timeout_ms: 5000
        }
      });
      
      setConsumeResult({
        loading: false,
        messages: response.data.messages || [],
        error: null
      });
    } catch (error) {
      setConsumeResult({
        loading: false,
        messages: [],
        error: error.response?.data?.message || error.message
      });
    }
  };

  const getExampleData = (type) => {
    switch (type) {
      case 'alert':
        return JSON.stringify({
          id: `alert-${Date.now()}`,
          level: 'info',
          message: 'Test alert message',
          timestamp: new Date().toISOString()
        }, null, 2);
      case 'edr':
        return JSON.stringify({
          device_id: 'device-123',
          threat_id: 'threat-456',
          process_name: 'test.exe',
          severity: 'medium',
          timestamp: new Date().toISOString()
        }, null, 2);
      case 'ngav':
        return JSON.stringify({
          alert_id: 'ngav-789',
          threat_type: 'malware',
          file_path: '/tmp/test.exe',
          blocked: true,
          timestamp: new Date().toISOString()
        }, null, 2);
      default:
        return '';
    }
  };

  return (
    <div className="app">
      <header className="app-header">
        <h1>🚨 Kafka Alerts Dashboard</h1>
        <p>Test Kafka connectivity and manage alert messages</p>
      </header>

      <main className="main-content">
        {/* Configuration Section */}
        <section className="card">
          <h2>📡 Configuration</h2>
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
              {connectivityTest.loading ? 'Testing...' : 'Test Default Config'}
            </button>
            
            <div className="custom-config">
              <h3>Custom Configuration</h3>
              <input
                type="text"
                placeholder="Bootstrap Servers"
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

        {/* Send Message Section */}
        <section className="card">
          <h2>📤 Send Message</h2>
          
          <div className="message-form">
            <div className="form-row">
              <label>Message Type:</label>
              <select
                value={messageForm.type}
                onChange={(e) => {
                  setMessageForm({
                    ...messageForm,
                    type: e.target.value,
                    data: getExampleData(e.target.value)
                  });
                }}
                className="select"
              >
                <option value="alert">Alert</option>
                <option value="edr">EDR Alert</option>
                <option value="ngav">NGAV Alert</option>
              </select>
            </div>
            
            <div className="form-row">
              <label>Topic (optional):</label>
              <input
                type="text"
                placeholder="Leave empty to use default"
                value={messageForm.topic}
                onChange={(e) => setMessageForm({
                  ...messageForm,
                  topic: e.target.value
                })}
                className="input"
              />
            </div>
            
            <div className="form-row">
              <label>Message Data (JSON):</label>
              <textarea
                value={messageForm.data}
                onChange={(e) => setMessageForm({
                  ...messageForm,
                  data: e.target.value
                })}
                className="textarea"
                rows="8"
                placeholder="Enter JSON data..."
              />
            </div>
            
            <button onClick={sendMessage} className="btn btn-primary">
              Send Message
            </button>
          </div>

          {sendResult && (
            <div className={`result ${sendResult.success ? 'success' : 'error'}`}>
              <h3>{sendResult.success ? '✅ Message Sent' : '❌ Send Failed'}</h3>
              <p>{sendResult.message}</p>
              {sendResult.message_id && (
                <p><strong>Message ID:</strong> {sendResult.message_id}</p>
              )}
            </div>
          )}
        </section>

        {/* Consume Messages Section */}
        <section className="card">
          <h2>📥 Consume Messages</h2>
          
          <button 
            onClick={consumeMessages}
            disabled={consumeResult.loading}
            className="btn btn-primary"
          >
            {consumeResult.loading ? 'Consuming...' : 'Consume Latest Messages'}
          </button>

          {consumeResult.error && (
            <div className="result error">
              <h3>❌ Error</h3>
              <p>{consumeResult.error}</p>
            </div>
          )}

          {consumeResult.messages.length > 0 && (
            <div className="messages-list">
              <h3>📨 Received Messages ({consumeResult.messages.length})</h3>
              {consumeResult.messages.map((msg, index) => (
                <div key={index} className="message-item">
                  <div className="message-meta">
                    <span>Partition: {msg.partition}</span>
                    <span>Offset: {msg.offset}</span>
                    {msg.timestamp && (
                      <span>Time: {new Date(msg.timestamp).toLocaleString()}</span>
                    )}
                    {msg.key && <span>Key: {msg.key}</span>}
                  </div>
                  <div className="message-payload">
                    <pre>{msg.payload}</pre>
                  </div>
                </div>
              ))}
            </div>
          )}
        </section>
      </main>
    </div>
  );
}

export default App;