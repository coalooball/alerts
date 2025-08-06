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
  const [correlationMode, setCorrelationMode] = useState('existing'); // 'existing' or 'new'
  const [threatEventSearch, setThreatEventSearch] = useState('');
  const [filteredThreatEvents, setFilteredThreatEvents] = useState([]);
  const [showThreatEventDropdown, setShowThreatEventDropdown] = useState(false);
  
  // 新威胁事件表单数据
  const [newThreatEvent, setNewThreatEvent] = useState({
    title: '',
    description: '',
    event_type: '',
    severity: 3,
    threat_category: '',
    priority: 'medium'
  });
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

  // 威胁事件搜索过滤
  useEffect(() => {
    if (threatEventSearch.trim() === '') {
      setFilteredThreatEvents([]);
      setShowThreatEventDropdown(false);
    } else {
      const filtered = threatEvents.filter(event => 
        event.title.toLowerCase().includes(threatEventSearch.toLowerCase()) ||
        event.id.toLowerCase().includes(threatEventSearch.toLowerCase()) ||
        event.event_type.toLowerCase().includes(threatEventSearch.toLowerCase())
      );
      setFilteredThreatEvents(filtered.slice(0, 10)); // 限制显示10个结果
      setShowThreatEventDropdown(filtered.length > 0);
    }
  }, [threatEventSearch, threatEvents]);

  // 点击外部关闭下拉框
  useEffect(() => {
    const handleClickOutside = (event) => {
      if (!event.target.closest('.search-dropdown-container')) {
        setShowThreatEventDropdown(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);

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
    if (correlationMode === 'existing') {
      if (!selectedThreatEvent || !correlationType) {
        if (window.showToast) {
          window.showToast('请选择威胁事件并输入关联类型', 'warning');
        }
        return;
      }
    } else {
      if (!newThreatEvent.title || !newThreatEvent.event_type || !correlationType) {
        if (window.showToast) {
          window.showToast('请填写威胁事件标题、类型和关联类型', 'warning');
        }
        return;
      }
    }

    try {
      const sessionToken = localStorage.getItem('sessionToken');
      let threatEventId = selectedThreatEvent;

      // 如果是新建威胁事件模式，先创建威胁事件
      if (correlationMode === 'new') {
        const createResponse = await fetch('/api/threat-events', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${sessionToken}`
          },
          body: JSON.stringify(newThreatEvent),
        });

        const createData = await createResponse.json();
        
        if (!createData.success) {
          if (window.showToast) {
            window.showToast('创建威胁事件失败: ' + createData.message, 'error');
          }
          return;
        }

        threatEventId = createData.threat_event_id;
      }

      // 关联告警到威胁事件
      const response = await fetch('/api/correlate-alerts', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${sessionToken}`
        },
        body: JSON.stringify({
          threat_event_id: threatEventId,
          alert_data_ids: selectedAlerts,
          correlation_type: correlationType,
          correlation_reason: correlationReason,
        }),
      });

      const data = await response.json();
      
      if (data.success) {
        if (window.showToast) {
          const mode = correlationMode === 'new' ? '创建新威胁事件并' : '';
          window.showToast(`成功${mode}关联 ${data.correlations_created} 个告警到威胁事件`, 'success');
        }
        handleResetForm();
        fetchAnnotatedAlerts();
        fetchThreatEvents(); // 刷新威胁事件列表
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

  const handleResetForm = () => {
    setShowCorrelationModal(false);
    setSelectedAlerts([]);
    setSelectedThreatEvent('');
    setCorrelationType('');
    setCorrelationReason('');
    setCorrelationMode('existing');
    setThreatEventSearch('');
    setShowThreatEventDropdown(false);
    setNewThreatEvent({
      title: '',
      description: '',
      event_type: '',
      severity: 3,
      threat_category: '',
      priority: 'medium'
    });
  };

  const renderCorrelationModal = () => {
    if (!showCorrelationModal) return null;

    return (
      <div className="modal-overlay" onClick={handleResetForm}>
        <div className="modal-content correlation-modal" onClick={(e) => e.stopPropagation()}>
          <div className="modal-header">
            <h3>关联告警到威胁事件</h3>
            <button className="close-button" onClick={handleResetForm}>
              ✕
            </button>
          </div>
          
          <div className="modal-body">
            <div className="correlation-info">
              <p>已选择 {selectedAlerts.length} 个告警进行关联</p>
            </div>

            <form className="correlation-form">
              {/* 关联模式选择 */}
              <div className="form-group">
                <label>关联方式 *:</label>
                <div className="radio-group">
                  <label className="radio-option">
                    <input
                      type="radio"
                      value="existing"
                      checked={correlationMode === 'existing'}
                      onChange={(e) => setCorrelationMode(e.target.value)}
                    />
                    <span>关联到已有威胁事件</span>
                  </label>
                  <label className="radio-option">
                    <input
                      type="radio"
                      value="new"
                      checked={correlationMode === 'new'}
                      onChange={(e) => setCorrelationMode(e.target.value)}
                    />
                    <span>创建新威胁事件</span>
                  </label>
                </div>
              </div>

              {/* 已有威胁事件搜索选择 */}
              {correlationMode === 'existing' && (
                <div className="form-group">
                  <label>威胁事件 *:</label>
                  <div className="search-dropdown-container">
                    <input
                      type="text"
                      className="form-input"
                      placeholder="搜索威胁事件标题、ID或类型..."
                      value={threatEventSearch}
                      onChange={(e) => setThreatEventSearch(e.target.value)}
                      onFocus={() => setShowThreatEventDropdown(filteredThreatEvents.length > 0)}
                      autoComplete="off"
                    />
                    {showThreatEventDropdown && (
                      <div className="search-dropdown">
                        {filteredThreatEvents.map((event, index) => (
                          <div
                            key={event.id || index}
                            className="dropdown-item"
                            onClick={() => {
                              setSelectedThreatEvent(event.id);
                              setThreatEventSearch(`${event.title} (${event.event_type})`);
                              setShowThreatEventDropdown(false);
                            }}
                          >
                            <div className="event-title">{event.title}</div>
                            <div className="event-details">
                              类型: {event.event_type} | 严重程度: {
                                event.severity === 1 ? '严重' : 
                                event.severity === 2 ? '高' : 
                                event.severity === 3 ? '中' : 
                                event.severity === 4 ? '低' : '信息'
                              }
                            </div>
                            <div className="event-id">ID: {event.id}</div>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              )}

              {/* 新威胁事件表单 */}
              {correlationMode === 'new' && (
                <>
                  <div className="form-group">
                    <label>事件标题 *:</label>
                    <input
                      type="text"
                      className="form-input"
                      placeholder="请输入威胁事件标题..."
                      value={newThreatEvent.title}
                      onChange={(e) => setNewThreatEvent({...newThreatEvent, title: e.target.value})}
                    />
                  </div>

                  <div className="form-group">
                    <label>事件描述:</label>
                    <textarea
                      className="form-textarea"
                      placeholder="详细描述威胁事件..."
                      rows="2"
                      value={newThreatEvent.description}
                      onChange={(e) => setNewThreatEvent({...newThreatEvent, description: e.target.value})}
                    />
                  </div>

                  <div className="form-row">
                    <div className="form-group">
                      <label>事件类型 *:</label>
                      <select
                        className="form-select"
                        value={newThreatEvent.event_type}
                        onChange={(e) => setNewThreatEvent({...newThreatEvent, event_type: e.target.value})}
                      >
                        <option value="">请选择事件类型</option>
                        <option value="malware">恶意软件</option>
                        <option value="persistence">持久化</option>
                        <option value="lateral_movement">横向移动</option>
                        <option value="data_exfiltration">数据泄露</option>
                        <option value="command_and_control">命令控制</option>
                        <option value="initial_access">初始访问</option>
                        <option value="privilege_escalation">权限提升</option>
                      </select>
                    </div>

                    <div className="form-group">
                      <label>严重程度 *:</label>
                      <select
                        className="form-select"
                        value={newThreatEvent.severity}
                        onChange={(e) => setNewThreatEvent({...newThreatEvent, severity: parseInt(e.target.value)})}
                      >
                        <option value={1}>严重</option>
                        <option value={2}>高</option>
                        <option value={3}>中</option>
                        <option value={4}>低</option>
                        <option value={5}>信息</option>
                      </select>
                    </div>
                  </div>

                  <div className="form-row">
                    <div className="form-group">
                      <label>威胁类别:</label>
                      <input
                        type="text"
                        className="form-input"
                        placeholder="例: APT, ransomware, trojan..."
                        value={newThreatEvent.threat_category}
                        onChange={(e) => setNewThreatEvent({...newThreatEvent, threat_category: e.target.value})}
                      />
                    </div>

                    <div className="form-group">
                      <label>优先级:</label>
                      <select
                        className="form-select"
                        value={newThreatEvent.priority}
                        onChange={(e) => setNewThreatEvent({...newThreatEvent, priority: e.target.value})}
                      >
                        <option value="critical">紧急</option>
                        <option value="high">高</option>
                        <option value="medium">中</option>
                        <option value="low">低</option>
                      </select>
                    </div>
                  </div>
                </>
              )}

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
            <button className="cancel-button" onClick={handleResetForm}>
              取消
            </button>
            <button 
              className="save-button" 
              onClick={handleSaveCorrelation}
              disabled={
                (correlationMode === 'existing' && (!selectedThreatEvent || !correlationType)) ||
                (correlationMode === 'new' && (!newThreatEvent.title || !newThreatEvent.event_type || !correlationType))
              }
            >
              {correlationMode === 'new' ? '创建事件并关联' : '确认关联'}
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