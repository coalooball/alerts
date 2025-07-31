import React from 'react';

const KafkaConfig = ({ 
  configs, 
  newConfig, 
  setNewConfig,
  editingConfig,
  showConfigForm, 
  setShowConfigForm,
  saveResult,
  connectivityTests,
  saveConfig,
  startEditConfig,
  deleteConfig,
  toggleConfigActive,
  testConnectivity,
  cancelEdit
}) => {
  return (
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
};

export default KafkaConfig;