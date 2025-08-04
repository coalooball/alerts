import React from 'react';
import KafkaDataChart from './KafkaDataChart';

const HomePage = ({ activeConfigs, configs, showTooltip, hideTooltip }) => {
  return (
    <div className="home-content">
      <KafkaDataChart />

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