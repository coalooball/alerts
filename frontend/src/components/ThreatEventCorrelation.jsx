import React, { useState, useEffect } from 'react';
import '../App.css';

const ThreatEventCorrelation = () => {
  const [annotatedAlerts, setAnnotatedAlerts] = useState([]);
  const [threatEvents, setThreatEvents] = useState([]);
  const [loading, setLoading] = useState(false);
  const [selectedAlerts, setSelectedAlerts] = useState([]);
  const [selectedThreatEvent, setSelectedThreatEvent] = useState('');
  const [correlationType, setCorrelationType] = useState('');
  const [correlationReason, setCorrelationReason] = useState('');
  const [showCorrelationModal, setShowCorrelationModal] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const [itemsPerPage] = useState(20);

  // 获取已标注的告警数据
  const fetchAnnotatedAlerts = async () => {
    setLoading(true);
    try {
      const sessionToken = localStorage.getItem('sessionToken');
      const params = new URLSearchParams();
      params.append('limit', itemsPerPage.toString());
      params.append('offset', ((currentPage - 1) * itemsPerPage).toString());
      
      const response = await fetch(`/api/annotations?${params.toString()}`, {
        headers: {
          'Authorization': `Bearer ${sessionToken}`
        }
      });
      const data = await response.json();
      
      if (data.success) {
        setAnnotatedAlerts(data.annotations);
      } else {
        console.error('Failed to fetch annotated alerts');
        setAnnotatedAlerts([]);
      }
    } catch (error) {
      console.error('Error fetching annotated alerts:', error);
      setAnnotatedAlerts([]);
    } finally {
      setLoading(false);
    }
  };

  // 获取威胁事件列表
  const fetchThreatEvents = async () => {
    try {
      const sessionToken = localStorage.getItem('sessionToken');
      const response = await fetch('/api/threat-events', {
        headers: {
          'Authorization': `Bearer ${sessionToken}`
        }
      });
      const data = await response.json();
      
      if (data.success) {
        setThreatEvents(data.threat_events);
      } else {
        console.error('Failed to fetch threat events');
        setThreatEvents([]);
      }
    } catch (error) {
      console.error('Error fetching threat events:', error);
      setThreatEvents([]);
    }
  };

  useEffect(() => {
    fetchAnnotatedAlerts();
    fetchThreatEvents();
  }, [currentPage]);

  const formatTimestamp = (timestamp) => {
    return new Date(timestamp).toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  };

  const getThreatLevelColor = (threatLevel) => {
    switch (threatLevel) {
      case 'critical': return '#dc2626';
      case 'high': return '#ea580c';
      case 'medium': return '#ca8a04';
      case 'low': return '#16a34a';
      case 'info': return '#6b7280';
      default: return '#6b7280';
    }
  };

  const getThreatLevelText = (threatLevel) => {
    switch (threatLevel) {
      case 'critical': return '严重';
      case 'high': return '高';
      case 'medium': return '中';
      case 'low': return '低';
      case 'info': return '信息';
      default: return '未知';
    }
  };

  const handleAlertSelection = (alertId) => {
    setSelectedAlerts(prev => {
      if (prev.includes(alertId)) {
        return prev.filter(id => id !== alertId);
      } else {
        return [...prev, alertId];
      }
    });
  };

  const handleCorrelateAlerts = () => {
    if (selectedAlerts.length === 0) {
      if (window.showToast) {
        window.showToast('请选择要关联的告警', 'warning');
      }
      return;
    }
    setShowCorrelationModal(true);
  };

  const handleSaveCorrelation = async () => {
    if (!selectedThreatEvent || !correlationType) {
      if (window.showToast) {
        window.showToast('请输入威胁事件和关联类型', 'warning');
      }
      return;
    }

    try {
      const sessionToken = localStorage.getItem('sessionToken');
      const response = await fetch('/api/correlate-alerts', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${sessionToken}`
        },
        body: JSON.stringify({
          threat_event_id: selectedThreatEvent,
          alert_data_ids: selectedAlerts,
          correlation_type: correlationType,
          correlation_reason: correlationReason,
        }),
      });

      const data = await response.json();
      
      if (data.success) {
        if (window.showToast) {
          window.showToast(`成功关联 ${data.correlations_created} 个告警到威胁事件`, 'success');
        }
        setShowCorrelationModal(false);
        setSelectedAlerts([]);
        setSelectedThreatEvent('');
        setCorrelationType('');
        setCorrelationReason('');
        fetchAnnotatedAlerts();
      } else {
        if (window.showToast) {
          window.showToast('关联失败: ' + data.message, 'error');
        }
      }
    } catch (error) {
      console.error('Error correlating alerts:', error);
      if (window.showToast) {
        window.showToast('关联告警时发生错误', 'error');
      }
    }
  };

  const renderCorrelationModal = () => {
    if (!showCorrelationModal) return null;

    return (
      <div className="modal-overlay" onClick={() => setShowCorrelationModal(false)}>
        <div className="modal-content correlation-modal" onClick={(e) => e.stopPropagation()}>
          <div className="modal-header">
            <h3>关联告警到威胁事件</h3>
            <button className="close-button" onClick={() => setShowCorrelationModal(false)}>
              ✕
            </button>
          </div>
          
          <div className="modal-body">
            <div className="correlation-info">
              <p>已选择 {selectedAlerts.length} 个告警进行关联</p>
            </div>

            <form className="correlation-form">
              <div className="form-group">
                <label>威胁事件 *:</label>
                <input
                  type="text"
                  className="form-input"
                  placeholder="请输入威胁事件名称或ID..."
                  value={selectedThreatEvent}
                  onChange={(e) => setSelectedThreatEvent(e.target.value)}
                />
              </div>

              <div className="form-group">
                <label>关联类型 *:</label>
                <input
                  type="text"
                  className="form-input"
                  placeholder="请输入关联类型 (如: 时间关联、行为关联、设备关联等)..."
                  value={correlationType}
                  onChange={(e) => setCorrelationType(e.target.value)}
                />
              </div>

              <div className="form-group">
                <label>关联原因:</label>
                <textarea
                  className="form-textarea"
                  placeholder="描述告警与威胁事件的关联原因..."
                  rows="3"
                  value={correlationReason}
                  onChange={(e) => setCorrelationReason(e.target.value)}
                />
              </div>
            </form>
          </div>

          <div className="modal-footer">
            <button className="cancel-button" onClick={() => setShowCorrelationModal(false)}>
              取消
            </button>
            <button 
              className="save-button" 
              onClick={handleSaveCorrelation}
              disabled={!selectedThreatEvent || !correlationType}
            >
              确认关联
            </button>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div className="threat-event-correlation-container">
      <div className="threat-event-correlation-header">
        <h3>威胁事件关联</h3>
        <div className="header-controls">
          <div className="pagination">
            <button 
              onClick={() => setCurrentPage(prev => Math.max(prev - 1, 1))}
              disabled={currentPage === 1}
              className="pagination-button"
            >
              上一页
            </button>
            <span className="page-info">第 {currentPage} 页</span>
            <button 
              onClick={() => setCurrentPage(prev => prev + 1)}
              disabled={annotatedAlerts.length < itemsPerPage}
              className="pagination-button"
            >
              下一页
            </button>
          </div>
          <button 
            className="correlate-button" 
            onClick={handleCorrelateAlerts}
            disabled={selectedAlerts.length === 0}
          >
            🔗 关联选中告警 ({selectedAlerts.length})
          </button>
          <button onClick={fetchAnnotatedAlerts} className="refresh-button" disabled={loading}>
            {loading ? '⏳ 加载中...' : '🔄 刷新'}
          </button>
        </div>
      </div>

      <div className="description-section">
        <p>📋 此页面显示所有已标注的告警数据，您可以选择多个告警并将它们关联到威胁事件中。</p>
      </div>

      {loading ? (
        <div className="loading-indicator">
          <div className="spinner"></div>
          <p>正在加载已标注告警...</p>
        </div>
      ) : (
        <div className="annotated-alerts-table-container">
          <table className="annotated-alerts-table">
            <thead>
              <tr>
                <th width="40">选择</th>
                <th>标注时间</th>
                <th>告警ID</th>
                <th>标注类型</th>
                <th>威胁等级</th>
                <th>恶意性</th>
                <th>MITRE技术</th>
                <th>标注人</th>
                <th>审核状态</th>
              </tr>
            </thead>
            <tbody>
              {annotatedAlerts.length === 0 ? (
                <tr>
                  <td colSpan="9" className="no-data">
                    暂无已标注的告警数据
                  </td>
                </tr>
              ) : (
                annotatedAlerts.map((annotation, index) => (
                  <tr key={annotation.id || index} className="annotation-row">
                    <td className="checkbox-cell">
                      <input
                        type="checkbox"
                        checked={selectedAlerts.includes(annotation.alert_data_id)}
                        onChange={() => handleAlertSelection(annotation.alert_data_id)}
                      />
                    </td>
                    <td className="timestamp-cell">
                      {formatTimestamp(annotation.annotated_at)}
                    </td>
                    <td className="alert-id-cell" title={annotation.alert_data_id}>
                      {annotation.alert_data_id.substring(0, 8)}...
                    </td>
                    <td className="type-cell">{annotation.annotation_type}</td>
                    <td className="threat-level-cell">
                      {annotation.threat_level ? (
                        <span 
                          className="threat-level-badge"
                          style={{ backgroundColor: getThreatLevelColor(annotation.threat_level) }}
                        >
                          {getThreatLevelText(annotation.threat_level)}
                        </span>
                      ) : '-'}
                    </td>
                    <td className="malicious-cell">
                      {annotation.is_malicious === null ? '-' : 
                       annotation.is_malicious ? '✅ 恶意' : '❌ 良性'}
                    </td>
                    <td className="mitre-cell">
                      {annotation.mitre_techniques && Array.isArray(annotation.mitre_techniques) ? 
                        annotation.mitre_techniques.join(', ') : '-'}
                    </td>
                    <td className="annotator-cell">{annotation.annotated_by_username || '-'}</td>
                    <td className="review-status-cell">
                      <span className={`review-status-badge ${annotation.review_status}`}>
                        {annotation.review_status === 'pending' ? '待审核' :
                         annotation.review_status === 'approved' ? '已通过' : '已拒绝'}
                      </span>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      )}

      {renderCorrelationModal()}
    </div>
  );
};

export default ThreatEventCorrelation;