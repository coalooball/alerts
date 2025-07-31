import React from 'react';

const TestModal = ({ testModal, closeTestModal }) => {
  if (!testModal.show) return null;

  const getModalTitle = () => {
    if (testModal.type === 'toggle') {
      return '⚙️ 配置状态切换';
    }
    return '🔗 连接测试结果';
  };

  const getLoadingMessage = () => {
    if (testModal.type === 'toggle') {
      return '正在测试连接并切换配置状态，请稍候...';
    }
    return '正在连接Kafka服务器，请稍候...';
  };

  return (
    <div className="modal-overlay" onClick={closeTestModal}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>{getModalTitle()}</h3>
          <button className="modal-close" onClick={closeTestModal}>
            ×
          </button>
        </div>
        
        <div className="modal-body">
          {testModal.config && (
            <div className="test-config-info">
              <h4>📋 {testModal.type === 'toggle' ? '操作配置信息' : '测试配置信息'}</h4>
              <div className="config-info-grid">
                <div><strong>配置名称：</strong>{testModal.config.name}</div>
                <div><strong>服务器地址：</strong>{testModal.config.bootstrap_servers}</div>
                <div><strong>主题名称：</strong>{testModal.config.topic}</div>
                <div><strong>消费者组：</strong>{testModal.config.group_id}</div>
              </div>
            </div>
          )}
          
          <div className="test-result-section">
            <h4>📊 {testModal.type === 'toggle' ? '操作结果' : '测试结果'}</h4>
            
            {testModal.loading && (
              <div className="test-loading">
                <div className="loading-spinner"></div>
                <span>{getLoadingMessage()}</span>
              </div>
            )}
            
            {testModal.result && (
              <div className={`test-result ${testModal.result.success ? 'success' : 'error'}`}>
                <div className="result-status">
                  {testModal.result.success ? (
                    <><span className="status-icon">✅</span> {testModal.type === 'toggle' ? '操作成功！' : '连接成功！'}</>
                  ) : (
                    <><span className="status-icon">❌</span> {testModal.type === 'toggle' ? '操作失败！' : '连接失败！'}</>
                  )}
                </div>
                
                <div className="result-message">
                  <strong>详细信息：</strong>
                  <p>{testModal.result.message}</p>
                </div>
                
                {testModal.result.details && (
                  <div className="result-details">
                    <strong>{testModal.type === 'toggle' ? '操作详情：' : '连接详情：'}</strong>
                    <pre>{JSON.stringify(testModal.result.details, null, 2)}</pre>
                  </div>
                )}
              </div>
            )}
            
            {testModal.error && (
              <div className="test-result error">
                <div className="result-status">
                  <span className="status-icon">❌</span> {testModal.type === 'toggle' ? '操作失败！' : '连接失败！'}
                </div>
                
                <div className="result-message">
                  <strong>错误信息：</strong>
                  <p>{testModal.error}</p>
                </div>
              </div>
            )}
          </div>
        </div>
        
        <div className="modal-footer">
          <button className="btn btn-secondary" onClick={closeTestModal}>
            关闭
          </button>
        </div>
      </div>
    </div>
  );
};

export default TestModal;