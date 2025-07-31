import React from 'react';

const HomePage = ({ activeConfigs, configs, showTooltip, hideTooltip }) => {
  return (
    <div className="home-content">
      <div className="welcome-section">
        <h2>欢迎使用挖掘告警系统</h2>
        <p>这是一个基于Kafka的安全告警处理系统，支持多种数据源的告警收集和处理。</p>
      </div>
      
      <div className="status-overview">
        <div className="status-card">
          <h3>当前活跃配置</h3>
          <div className="status-number">{activeConfigs.length}</div>
          <p>个Kafka配置正在运行</p>
        </div>
        
        <div className="status-card">
          <h3>总配置数量</h3>
          <div className="status-number">{configs.length}</div>
          <p>个Kafka配置已创建</p>
        </div>
      </div>

      {activeConfigs.length > 0 && (
        <div className="active-configs-section">
          <h3>🟢 当前活跃的Kafka配置</h3>
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
                <span className="config-server">{cfg.bootstrap_servers}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default HomePage;