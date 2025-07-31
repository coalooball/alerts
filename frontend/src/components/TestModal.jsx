import React from 'react';

const TestModal = ({ testModal, closeTestModal }) => {
  if (!testModal.show) return null;

  return (
    <div className="modal-overlay" onClick={closeTestModal}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>🔗 连接测试结果</h3>
          <button className="modal-close" onClick={closeTestModal}>
            ×
          </button>
        </div>
        
        {testModal.config && (
          <div className="modal-body">
            <div className="test-config-info">
              <h4>📋 测试配置信息</h4>
              <div className="config-info-grid">
                <div><strong>配置名称：</strong>{testModal.config.name}</div>
                <div><strong>服务器地址：</strong>{testModal.config.bootstrap_servers}</div>
                <div><strong>主题名称：</strong>{testModal.config.topic}</div>
                <div><strong>消费者组：</strong>{testModal.config.group_id}</div>
              </div>
            </div>
            
            <div className="test-result-section">
              <h4>📊 测试结果</h4>
              
              {testModal.loading && (
                <div className="test-loading">
                  <div className="loading-spinner"></div>
                  <span>正在连接Kafka服务器，请稍候...</span>
                </div>
              )}
              
              {testModal.result && (
                <div className={`test-result ${testModal.result.success ? 'success' : 'error'}`}>
                  <div className="result-status">
                    {testModal.result.success ? (
                      <><span className="status-icon">✅</span> 连接成功！</>
                    ) : (
                      <><span className="status-icon">❌</span> 连接失败！</>
                    )}
                  </div>
                  
                  <div className="result-message">
                    <strong>详细信息：</strong>
                    <p>{testModal.result.message}</p>
                  </div>
                  
                  {testModal.result.details && (
                    <div className="result-details">
                      <strong>连接详情：</strong>
                      <pre>{JSON.stringify(testModal.result.details, null, 2)}</pre>
                    </div>
                  )}
                </div>
              )}
              
              {testModal.error && (
                <div className="test-result error">
                  <div className="result-status">
                    <span className="status-icon">❌</span> 连接失败！
                  </div>
                  
                  <div className="result-message">
                    <strong>错误信息：</strong>
                    <p>{testModal.error}</p>
                  </div>
                </div>
              )}
            </div>
          </div>
        )}
        
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