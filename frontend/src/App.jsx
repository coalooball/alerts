import React, { useState } from 'react';
import './App.css';

// 导入自定义hooks
import { useConfigs } from './hooks/useConfigs';
import { useConnectivity } from './hooks/useConnectivity';
import { useTooltip } from './hooks/useTooltip';

// 导入组件
import HomePage from './components/HomePage';
import KafkaConfig from './components/KafkaConfig';
import ClickHouseConfig from './components/ClickHouseConfig';
import TestModal from './components/TestModal';
import Tooltip from './components/Tooltip';

function App() {
  const [activeView, setActiveView] = useState('home');
  const [configTab, setConfigTab] = useState('kafka');

  // 使用自定义hooks
  const {
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
  } = useConfigs();

  const {
    connectivityTests,
    testModal,
    testConnectivity,
    testClickhouseConnectivity,
    toggleConfigActive,
    closeTestModal
  } = useConnectivity();

  const { tooltip, showTooltip, hideTooltip } = useTooltip();

  // 包装一些函数以传递依赖
  const handleSaveConfig = () => saveConfig(configTab);
  const handleToggleConfigActive = (configId, isActive) => 
    toggleConfigActive(configId, isActive, configs, testConnectivity, loadConfigs, loadActiveConfigs);

  const renderConfigPage = () => (
    <div className="config-content">
      <div className="config-header">
        <h2>配置管理</h2>
        <div className="config-tabs">
          <button 
            className={`tab-button ${configTab === 'kafka' ? 'active' : ''}`}
            onClick={() => setConfigTab('kafka')}
          >
            Kafka配置
          </button>
          <button 
            className={`tab-button ${configTab === 'clickhouse' ? 'active' : ''}`}
            onClick={() => setConfigTab('clickhouse')}
          >
            ClickHouse配置
          </button>
        </div>
      </div>

      {configTab === 'kafka' && (
        <KafkaConfig
          configs={configs}
          newConfig={newConfig}
          setNewConfig={setNewConfig}
          editingConfig={editingConfig}
          showConfigForm={showConfigForm}
          setShowConfigForm={setShowConfigForm}
          saveResult={saveResult}
          connectivityTests={connectivityTests}
          saveConfig={handleSaveConfig}
          startEditConfig={startEditConfig}
          deleteConfig={deleteConfig}
          toggleConfigActive={handleToggleConfigActive}
          testConnectivity={testConnectivity}
          cancelEdit={cancelEdit}
        />
      )}

      {configTab === 'clickhouse' && (
        <ClickHouseConfig
          clickhouseConfig={clickhouseConfig}
          newClickhouseConfig={newClickhouseConfig}
          setNewClickhouseConfig={setNewClickhouseConfig}
          showConfigForm={showConfigForm}
          setShowConfigForm={setShowConfigForm}
          saveResult={saveResult}
          saveConfig={handleSaveConfig}
          testClickhouseConnectivity={testClickhouseConnectivity}
        />
      )}
    </div>
  );

  return (
    <div className="app">
      <header className="app-header">
        <h1>挖掘告警系统</h1>
      </header>

      <div className="app-container">
        <nav className="sidebar">
          <ul className="nav-menu">
            <li className={activeView === 'home' ? 'active' : ''}>
              <button onClick={() => setActiveView('home')}>
                🏠 首页
              </button>
            </li>
            <li className={activeView === 'config' ? 'active' : ''}>
              <button onClick={() => setActiveView('config')}>
                ⚙️ 配置
              </button>
            </li>
          </ul>
        </nav>

        <main className="main-content">
          {activeView === 'home' && (
            <HomePage
              activeConfigs={activeConfigs}
              configs={configs}
              showTooltip={showTooltip}
              hideTooltip={hideTooltip}
            />
          )}
          {activeView === 'config' && renderConfigPage()}
        </main>
      </div>

      <Tooltip tooltip={tooltip} />
      <TestModal testModal={testModal} closeTestModal={closeTestModal} />
    </div>
  );
}

export default App;