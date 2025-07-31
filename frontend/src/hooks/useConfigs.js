import { useState, useEffect } from 'react';
import axios from 'axios';

export const useConfigs = () => {
  const [configs, setConfigs] = useState([]);
  const [activeConfigs, setActiveConfigs] = useState([]);
  const [clickhouseConfig, setClickhouseConfig] = useState(null);
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
  const [newClickhouseConfig, setNewClickhouseConfig] = useState({
    name: 'default',
    host: '10.26.64.224',
    port: 8123,
    database_name: 'default',
    username: 'default',
    password: 'default',
    use_tls: false,
    connection_timeout_ms: 10000,
    request_timeout_ms: 30000,
    max_connections: 10
  });
  const [editingConfig, setEditingConfig] = useState(null);
  const [showConfigForm, setShowConfigForm] = useState(false);
  const [saveResult, setSaveResult] = useState(null);

  useEffect(() => {
    loadConfigs();
    loadActiveConfigs();
    loadClickhouseConfig();
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

  const loadClickhouseConfig = async () => {
    try {
      const response = await axios.get('/api/clickhouse-config');
      if (response.data.success && response.data.config) {
        setClickhouseConfig(response.data.config);
        setNewClickhouseConfig(response.data.config);
      }
    } catch (error) {
      console.error('Failed to load ClickHouse config:', error);
    }
  };

  const saveConfig = async (configTab) => {
    setSaveResult(null);
    
    try {
      let response;
      if (configTab === 'clickhouse') {
        response = await axios.post('/api/clickhouse-config', newClickhouseConfig);
      } else {
        if (editingConfig) {
          response = await axios.put(`/api/config/${editingConfig.id}`, newConfig);
        } else {
          response = await axios.post('/api/config', newConfig);
        }
      }
      
      setSaveResult(response.data);
      if (response.data.success) {
        setShowConfigForm(false);
        setEditingConfig(null);
        if (configTab === 'clickhouse') {
          loadClickhouseConfig();
        } else {
          loadConfigs();
          loadActiveConfigs();
          resetConfigForm();
        }
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

  const cancelEdit = () => {
    setShowConfigForm(false);
    setEditingConfig(null);
    setSaveResult(null);
    resetConfigForm();
  };

  return {
    configs,
    activeConfigs,
    clickhouseConfig,
    newConfig,
    setNewConfig,
    newClickhouseConfig,
    setNewClickhouseConfig,
    editingConfig,
    showConfigForm,
    setShowConfigForm,
    saveResult,
    loadConfigs,
    loadActiveConfigs,
    saveConfig,
    startEditConfig,
    deleteConfig,
    cancelEdit
  };
};