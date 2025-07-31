import { useState } from 'react';
import axios from 'axios';

export const useConnectivity = () => {
  const [connectivityTests, setConnectivityTests] = useState({});
  const [testModal, setTestModal] = useState({ 
    show: false, 
    loading: false, 
    result: null, 
    error: null, 
    config: null 
  });

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

  const testClickhouseConnectivity = async (showModal = false) => {
    if (showModal) {
      setTestModal({
        show: true,
        loading: true,
        result: null,
        error: null,
        config: null // ClickHouse doesn't need config display in modal
      });
    }
    
    try {
      const response = await axios.get('/api/test-clickhouse-connectivity');
      
      if (showModal) {
        setTestModal(prev => ({
          ...prev,
          loading: false,
          result: response.data,
          error: null
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
      }
      
      return false;
    }
  };

  const toggleConfigActive = async (configId, isActive, configs, testConnectivity, loadConfigs, loadActiveConfigs) => {
    if (isActive) {
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

  return {
    connectivityTests,
    testModal,
    testConnectivity,
    testClickhouseConnectivity,
    toggleConfigActive,
    closeTestModal
  };
};