import React from 'react';

const HomePage = ({ activeConfigs, configs, showTooltip, hideTooltip }) => {
  return (
    <div className="home-content">
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
          <div className="active-configs-inline">
            {activeConfigs.map((cfg, index) => (
              <React.Fragment key={cfg.id}>
                {index > 0 && <span className="separator"> | </span>}
                <span 
                  className="config-item"
                  onMouseEnter={(e) => showTooltip(e, cfg)}
                  onMouseLeave={hideTooltip}
                >
                  <strong>{cfg.name}</strong>: {cfg.bootstrap_servers} → {cfg.topic}
                </span>
              </React.Fragment>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default HomePage;