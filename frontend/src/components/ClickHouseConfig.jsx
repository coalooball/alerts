import React from 'react';

const ClickHouseConfig = ({ 
  clickhouseConfig,
  newClickhouseConfig,
  setNewClickhouseConfig,
  showConfigForm,
  setShowConfigForm,
  saveResult,
  saveConfig,
  testClickhouseConnectivity
}) => {
  return (
    <div className="clickhouse-config-content">
      <div className="config-header">
        <h3>ClickHouse配置管理</h3>
        <div className="config-actions">
          <button 
            onClick={() => setShowConfigForm(!showConfigForm)}
            className="btn btn-primary"
          >
            {showConfigForm ? '取消' : clickhouseConfig ? '编辑配置' : '新建配置'}
          </button>
          {clickhouseConfig && (
            <button 
              onClick={() => testClickhouseConnectivity(true)}
              className="btn btn-info"
            >
              测试连接
            </button>
          )}
        </div>
      </div>

      {showConfigForm && (
        <div className="config-form">
          <h4>{clickhouseConfig ? '编辑ClickHouse配置' : '创建ClickHouse配置'}</h4>
          <div className="form-grid">
            <div className="form-row">
              <label>配置名称：</label>
              <input
                type="text"
                value={newClickhouseConfig.name}
                onChange={(e) => setNewClickhouseConfig({...newClickhouseConfig, name: e.target.value})}
                className="input"
                placeholder="输入配置名称"
              />
            </div>
            
            <div className="form-row">
              <label>主机地址：</label>
              <input
                type="text"
                value={newClickhouseConfig.host}
                onChange={(e) => setNewClickhouseConfig({...newClickhouseConfig, host: e.target.value})}
                className="input"
                placeholder="例如：10.26.64.224"
              />
            </div>
            
            <div className="form-row">
              <label>端口：</label>
              <input
                type="number"
                value={newClickhouseConfig.port}
                onChange={(e) => setNewClickhouseConfig({...newClickhouseConfig, port: parseInt(e.target.value)})}
                className="input"
                placeholder="8123"
              />
            </div>
            
            <div className="form-row">
              <label>数据库名称：</label>
              <input
                type="text"
                value={newClickhouseConfig.database_name}
                onChange={(e) => setNewClickhouseConfig({...newClickhouseConfig, database_name: e.target.value})}
                className="input"
                placeholder="例如：alerts"
              />
            </div>

            <div className="form-row">
              <label>用户名：</label>
              <input
                type="text"
                value={newClickhouseConfig.username}
                onChange={(e) => setNewClickhouseConfig({...newClickhouseConfig, username: e.target.value})}
                className="input"
                placeholder="default"
              />
            </div>

            <div className="form-row">
              <label>密码：</label>
              <input
                type="password"
                value={newClickhouseConfig.password || ''}
                onChange={(e) => setNewClickhouseConfig({...newClickhouseConfig, password: e.target.value})}
                className="input"
                placeholder="留空表示无密码"
              />
            </div>

            <div className="form-row">
              <label>
                <input
                  type="checkbox"
                  checked={newClickhouseConfig.use_tls}
                  onChange={(e) => setNewClickhouseConfig({...newClickhouseConfig, use_tls: e.target.checked})}
                />
                使用TLS连接
              </label>
            </div>

            <div className="form-row">
              <label>连接超时时间（毫秒）：</label>
              <input
                type="number"
                value={newClickhouseConfig.connection_timeout_ms}
                onChange={(e) => setNewClickhouseConfig({...newClickhouseConfig, connection_timeout_ms: parseInt(e.target.value)})}
                className="input"
              />
            </div>

            <div className="form-row">
              <label>请求超时时间（毫秒）：</label>
              <input
                type="number"
                value={newClickhouseConfig.request_timeout_ms}
                onChange={(e) => setNewClickhouseConfig({...newClickhouseConfig, request_timeout_ms: parseInt(e.target.value)})}
                className="input"
              />
            </div>

            <div className="form-row">
              <label>最大连接数：</label>
              <input
                type="number"
                value={newClickhouseConfig.max_connections}
                onChange={(e) => setNewClickhouseConfig({...newClickhouseConfig, max_connections: parseInt(e.target.value)})}
                className="input"
              />
            </div>
          </div>
          
          <button onClick={saveConfig} className="btn btn-primary">
            {clickhouseConfig ? '更新配置' : '保存配置'}
          </button>

          {saveResult && (
            <div className={`result ${saveResult.success ? 'success' : 'error'}`}>
              <p>{saveResult.message}</p>
            </div>
          )}
        </div>
      )}

      {clickhouseConfig && (
        <div className="current-config">
          <h4>当前ClickHouse配置</h4>
          <div className="config-card">
            <div className="config-details">
              <p><strong>配置名称：</strong> {clickhouseConfig.name}</p>
              <p><strong>主机地址：</strong> {clickhouseConfig.host}</p>
              <p><strong>端口：</strong> {clickhouseConfig.port}</p>
              <p><strong>数据库：</strong> {clickhouseConfig.database_name}</p>
              <p><strong>用户名：</strong> {clickhouseConfig.username}</p>
              <p><strong>使用TLS：</strong> {clickhouseConfig.use_tls ? '是' : '否'}</p>
              <p><strong>连接超时：</strong> {clickhouseConfig.connection_timeout_ms}ms</p>
              <p><strong>请求超时：</strong> {clickhouseConfig.request_timeout_ms}ms</p>
              <p><strong>最大连接数：</strong> {clickhouseConfig.max_connections}</p>
              <p><strong>创建时间：</strong> {new Date(clickhouseConfig.created_at).toLocaleDateString()}</p>
            </div>
            
            <div className="config-actions">
              <button 
                onClick={() => setShowConfigForm(true)}
                className="btn btn-secondary btn-sm"
              >
                编辑
              </button>
              
              <button 
                onClick={() => testClickhouseConnectivity(true)}
                className="btn btn-info btn-sm"
              >
                测试连接
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default ClickHouseConfig;