import { useState } from 'react';
import axios from 'axios';

export const useConnectivity = () => {
  const [connectivityTests, setConnectivityTests] = useState({});
  const [testModal, setTestModal] = useState({ 
    show: false, 
    loading: false, 
    result: null, 
    error: null, 
    config: null,
    type: 'test' // 'test', 'toggle'
  });

  const testConnectivity = async (config, showModal = false) => {
    const configId = config.id;
    
    if (showModal) {
      setTestModal({
        show: true,
        loading: true,
        result: null,
        error: null,
        config: config,
        type: 'test'
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
        config: null, // ClickHouse doesn't need config display in modal
        type: 'test'
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
    const config = configs.find(c => c.id === configId);
    
    if (isActive && config) {
      // 显示配置切换的模态框
      setTestModal({
        show: true,
        loading: true,
        result: null,
        error: null,
        config: config,
        type: 'toggle'
      });

      // 静默测试连接（不显示连接测试结果）
      const connectionSuccess = await testConnectivity(config, false);
      
      if (!connectionSuccess) {
        // 连接测试失败，显示错误信息
        setTestModal(prev => ({
          ...prev,
          loading: false,
          result: null,
          error: '连接测试失败，无法激活配置！请检查配置信息后重试。',
          type: 'toggle'
        }));
        return;
      }
    } else if (!isActive && config) {
      // 停用配置时也显示模态框
      setTestModal({
        show: true,
        loading: true,
        result: null,
        error: null,
        config: config,
        type: 'toggle'
      });
    }

    try {
      // 尝试切换配置状态
      const response = await axios.post(`/api/config/${configId}/toggle`, { is_active: isActive });
      
      if (response.data.success) {
        loadConfigs();
        loadActiveConfigs();
        
        // 显示成功信息
        if (config) {
          setTestModal({
            show: true,
            loading: false,
            result: {
              success: true,
              message: isActive ? `配置 "${config.name}" 已成功激活` : `配置 "${config.name}" 已成功停用`
            },
            error: null,
            config: config,
            type: 'toggle'
          });
        }
      } else {
        // 显示失败信息
        setTestModal({
          show: true,
          loading: false,
          result: null,
          error: response.data.message || '操作失败，请重试',
          config: config,
          type: 'toggle'
        });
      }
    } catch (error) {
      console.error('Failed to toggle config:', error);
      // 显示错误信息
      setTestModal({
        show: true,
        loading: false,
        result: null,
        error: error.response?.data?.message || error.message || '网络错误，请检查连接后重试',
        config: config,
        type: 'toggle'
      });
    }
  };

  const closeTestModal = () => {
    setTestModal({ show: false, loading: false, result: null, error: null, config: null, type: 'test' });
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