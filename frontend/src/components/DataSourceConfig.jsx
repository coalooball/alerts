import React, { useState, useEffect } from 'react';
import axios from 'axios';

const DataSourceConfig = ({ configs }) => {
  const [dataSourceConfigs, setDataSourceConfigs] = useState({
    edr: { kafkaNodes: [] },
    ngav: { kafkaNodes: [] }
  });
  
  const [showDataFormats, setShowDataFormats] = useState(false);
  const [configModal, setConfigModal] = useState({
    isOpen: false,
    dataType: null,
    selectedNodes: []
  });
  const [loading, setLoading] = useState(false);
  const [saveResult, setSaveResult] = useState(null);

  // Load existing data source configurations on component mount
  useEffect(() => {
    loadDataSourceConfigs();
  }, []);

  // EDR 数据格式样例
  const edrSample = {
    "schema": 1,
    "create_time": "2022-07-19T17:48:25.018Z",
    "device_external_ip": "130.126.255.183",
    "device_id": 98483951,
    "device_internal_ip": "192.168.223.128",
    "device_name": "WIN-32-H1",
    "device_os": "WINDOWS",
    "ioc_hit": "(((process_cmdline:.ps* OR process_cmdline:.bat OR process_cmdline:.py...))",
    "ioc_id": "565644-0",
    "org_key": "7DMF69PK",
    "parent_cmdline": "C:\\Windows\\Explorer.EXE",
    "parent_guid": "7DMF69PK-05e5f750-00000608-00000000-1d89b87dc95da45",
    "parent_hash": ["40d777b7a95e00593eb1568c68514493"],
    "parent_path": "c:\\windows\\explorer.exe",
    "parent_pid": 1544,
    "parent_publisher": [{"name": "Microsoft Windows", "state": "FILE_SIGNATURE_STATE_SIGNED"}],
    "parent_reputation": "REP_WHITE",
    "parent_username": "WIN-32-H1\\aalsahee",
    "process_cmdline": "C:\\Windows\\system32\\cmd.exe /c \"start_dns_logs.bat\"",
    "process_guid": "7DMF69PK-05e5f750-00000b38-00000000-1d89b88d8c47a1c",
    "process_hash": ["ad7b9c14083b52bc532fba5948342b98"],
    "process_path": "c:\\windows\\system32\\cmd.exe",
    "process_pid": 2872,
    "process_publisher": [{"name": "Microsoft Windows", "state": "FILE_SIGNATURE_STATE_SIGNED"}],
    "process_reputation": "REP_WHITE",
    "process_username": "WIN-32-H1\\aalsahee",
    "report_id": "CFnKBKLTv6hUkBGFobRdg-565644",
    "report_name": "Execution - Command and Scripting Interpreter Execution",
    "report_tags": ["attack", "attackframework", "threathunting", "hunting", "windows"],
    "severity": 1,
    "type": "watchlist.hit",
    "watchlists": [{"id": "2HDA4XeT6uAvo3nY0oDA", "name": "ATT&CK Framework"}]
  };

  // NGAV 数据格式样例
  const ngavSample = {
    "type": "CB_ANALYTICS",
    "id": "15bcc53d-2f2e-9c70-c4ed-62bdb8b96e21",
    "legacy_alert_id": "15bcc53d-2f2e-9c70-c4ed-62bdb8b96e21",
    "org_key": "7DMF69PK",
    "create_time": "2022-07-19T16:04:27Z",
    "last_update_time": "2022-07-19T16:04:38Z",
    "first_event_time": "2022-07-19T16:03:00Z",
    "last_event_time": "2022-07-19T16:03:00Z",
    "threat_id": "47724a5ed183807a762e8ddae4cd490f",
    "severity": 3,
    "category": "NOTICE",
    "device_id": 98483951,
    "device_os": "WINDOWS",
    "device_os_version": "Windows 7 x86 SP: 1",
    "device_name": "WIN-32-H1",
    "device_username": "sts.victim@gmail.com",
    "policy_id": 268058,
    "policy_name": "Standard",
    "target_value": "MEDIUM",
    "workflow": {
      "state": "OPEN",
      "remediation": "",
      "last_update_time": "2022-07-19T16:04:27Z",
      "comment": "",
      "changed_by": "Carbon Black"
    },
    "device_internal_ip": "192.168.223.128",
    "device_external_ip": "130.126.255.183",
    "alert_url": "https://defense-prod05.conferdeploy.net/triage?incidentId=15bcc53d...",
    "reason": "The application python.exe acted as a network server.",
    "reason_code": "R_NET_SERVER",
    "process_name": "python.exe",
    "device_location": "OFFSITE",
    "created_by_event_id": "75c1fdb3077c11edb5d90fa3b0dbbdf7",
    "threat_indicators": [
      {
        "process_name": "python.exe",
        "sha256": "39cf1a29bbd79fcf84dbc6b765a1738e1a5b237a4cccef41684896075a990d30",
        "ttps": ["FIXED_PORT_LISTEN"]
      }
    ],
    "threat_cause_actor_sha256": "39cf1a29bbd79fcf84dbc6b765a1738e1a5b237a4cccef41684896075a990d30",
    "threat_cause_actor_name": "python.exe",
    "threat_cause_reputation": "TRUSTED_WHITE_LIST",
    "threat_cause_threat_category": "NON_MALWARE",
    "threat_cause_vector": "UNKNOWN",
    "blocked_threat_category": "UNKNOWN",
    "not_blocked_threat_category": "NON_MALWARE",
    "kill_chain_status": ["INSTALL_RUN"],
    "run_state": "RAN",
    "policy_applied": "NOT_APPLIED"
  };

  const loadDataSourceConfigs = async () => {
    try {
      setLoading(true);
      const response = await axios.get('/api/data-source-config');
      if (response.data.success && response.data.configs) {
        const configsByType = response.data.configs;
        const newConfigs = {
          edr: { kafkaNodes: [] },
          ngav: { kafkaNodes: [] }
        };

        // Parse the loaded configurations
        Object.keys(configsByType).forEach(dataType => {
          if (configsByType[dataType]) {
            newConfigs[dataType] = {
              kafkaNodes: configsByType[dataType].map(config => config.kafka_config_id)
            };
          }
        });

        setDataSourceConfigs(newConfigs);
      }
    } catch (error) {
      console.error('Failed to load data source configurations:', error);
    } finally {
      setLoading(false);
    }
  };

  const openConfigModal = (dataType) => {
    setConfigModal({
      isOpen: true,
      dataType,
      selectedNodes: [...dataSourceConfigs[dataType].kafkaNodes]
    });
    setSaveResult(null);
  };

  const closeConfigModal = () => {
    setConfigModal({
      isOpen: false,
      dataType: null,
      selectedNodes: []
    });
    setSaveResult(null);
  };

  const handleModalNodeChange = (nodeId, checked) => {
    setConfigModal(prev => ({
      ...prev,
      selectedNodes: checked 
        ? [...prev.selectedNodes, nodeId]
        : prev.selectedNodes.filter(id => id !== nodeId)
    }));
  };

  const saveModalConfig = async () => {
    const { dataType, selectedNodes } = configModal;
    
    try {
      setLoading(true);
      const response = await axios.post('/api/data-source-config', {
        data_type: dataType,
        kafka_config_ids: selectedNodes
      });

      if (response.data.success) {
        // Update local state
        setDataSourceConfigs(prev => ({
          ...prev,
          [dataType]: { kafkaNodes: selectedNodes }
        }));
        
        setSaveResult({
          success: true,
          message: response.data.message
        });

        // Close modal after a short delay to show success message
        setTimeout(() => {
          closeConfigModal();
        }, 1000);
      } else {
        setSaveResult({
          success: false,
          message: response.data.message
        });
      }
    } catch (error) {
      setSaveResult({
        success: false,
        message: error.response?.data?.message || error.message
      });
    } finally {
      setLoading(false);
    }
  };

  const getDataTypeName = (type) => {
    const names = {
      edr: 'EDR (Endpoint Detection and Response)',
      ngav: 'NGAV (Next-Gen Antivirus)'
    };
    return names[type] || type.toUpperCase();
  };

  const getSelectedNodesInfo = (dataType) => {
    const selectedNodeIds = dataSourceConfigs[dataType].kafkaNodes;
    const selectedConfigs = configs.filter(config => selectedNodeIds.includes(config.id));
    return selectedConfigs;
  };

  return (
    <div className="datasource-config-content">
      <div className="config-header">
        <h2>数据来源配置</h2>
        <p className="config-description">为不同的数据类型配置 Kafka 数据源节点</p>
        {loading && <div className="loading-indicator">加载中...</div>}
      </div>

      <div className="data-types-grid">
        {['edr', 'ngav'].map((dataType) => {
          const selectedNodes = getSelectedNodesInfo(dataType);
          return (
            <div key={dataType} className="data-type-card">
              <div className="data-type-header">
                <h3>{getDataTypeName(dataType)}</h3>
                <button 
                  onClick={() => openConfigModal(dataType)}
                  className="btn btn-primary btn-sm"
                >
                  配置数据源
                </button>
              </div>
              
              <div className="data-type-info">
                <div className="selected-nodes-summary">
                  <span className="node-count">
                    已选择 {selectedNodes.length} 个 Kafka 节点
                  </span>
                  {selectedNodes.length > 0 && (
                    <div className="selected-nodes-list">
                      {selectedNodes.map((node) => (
                        <div key={node.id} className="selected-node-item">
                          <span className="node-name">{node.name}</span>
                          <span className="node-server">{node.bootstrap_servers}</span>
                          {node.is_active && <span className="active-badge">活跃</span>}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            </div>
          );
        })}
      </div>

      <div className="form-actions">
        <button 
          onClick={() => setShowDataFormats(!showDataFormats)} 
          className="btn btn-info"
        >
          {showDataFormats ? '隐藏' : '查看'}数据格式
        </button>
      </div>

      {showDataFormats && (
        <div className="data-formats-section">
          <h3>数据格式说明</h3>
          
          <div className="data-format-tabs">
            <div className="data-format-item">
              <h4>EDR 数据格式</h4>
              <div className="json-viewer">
                <pre>{JSON.stringify(edrSample, null, 2)}</pre>
              </div>
            </div>
            
            <div className="data-format-item">
              <h4>NGAV 数据格式</h4>
              <div className="json-viewer">
                <pre>{JSON.stringify(ngavSample, null, 2)}</pre>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 数据源配置模态框 */}
      {configModal.isOpen && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h3>配置 {getDataTypeName(configModal.dataType)} 数据源</h3>
              <button onClick={closeConfigModal} className="modal-close-btn">×</button>
            </div>
            
            <div className="modal-body">
              {configs.length === 0 ? (
                <div className="no-kafka-nodes">
                  <p>没有可用的 Kafka 配置，请先在 Kafka 配置页面创建配置。</p>
                </div>
              ) : (
                <div className="kafka-nodes-list">
                  <p className="modal-description">选择用于 {getDataTypeName(configModal.dataType)} 数据的 Kafka 节点：</p>
                  {configs.map((config) => (
                    <div key={config.id} className="kafka-node-item">
                      <label className="checkbox-label">
                        <input
                          type="checkbox"
                          checked={configModal.selectedNodes.includes(config.id)}
                          onChange={(e) => handleModalNodeChange(config.id, e.target.checked)}
                        />
                        <div className="kafka-node-info">
                          <strong>{config.name}</strong>
                          <span className="kafka-node-details">
                            {config.bootstrap_servers} - {config.topic}
                            {config.is_active && <span className="active-badge">活跃</span>}
                          </span>
                        </div>
                      </label>
                    </div>
                  ))}
                </div>
              )}
            </div>
            
            <div className="modal-footer">
              {saveResult && (
                <div className={`save-result ${saveResult.success ? 'success' : 'error'}`}>
                  {saveResult.message}
                </div>
              )}
              <div className="modal-buttons">
                <button onClick={closeConfigModal} className="btn btn-secondary">
                  取消
                </button>
                <button 
                  onClick={saveModalConfig} 
                  className="btn btn-primary"
                  disabled={configs.length === 0 || loading}
                >
                  {loading ? '保存中...' : '保存配置'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default DataSourceConfig;