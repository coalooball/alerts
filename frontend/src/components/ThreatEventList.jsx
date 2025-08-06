import React, { useState, useEffect } from 'react';
import '../App.css';
import { useAuth } from '../contexts/AuthContext';

const ThreatEventList = () => {
  const { sessionToken } = useAuth();
  const [threatEvents, setThreatEvents] = useState([]);
  const [loading, setLoading] = useState(false);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [selectedEvent, setSelectedEvent] = useState(null);
  const [showDetailModal, setShowDetailModal] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const [itemsPerPage] = useState(20);
  const [searchFilters, setSearchFilters] = useState({
    status: '',
    severity: '',
    event_type: '',
    creation_method: '',
    date_from: '',
    date_to: ''
  });

  // 新建威胁事件的表单数据
  const [newEvent, setNewEvent] = useState({
    title: '',
    description: '',
    event_type: '',
    severity: 3,
    threat_category: '',
    event_start_time: '',
    event_end_time: '',
    mitre_tactics: [],
    mitre_techniques: [],
    priority: 'medium',
    tags: []
  });

  // 获取威胁事件列表
  const fetchThreatEvents = async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams();
      params.append('limit', itemsPerPage.toString());
      params.append('offset', ((currentPage - 1) * itemsPerPage).toString());
      
      Object.entries(searchFilters).forEach(([key, value]) => {
        if (value) {
          params.append(key, value);
        }
      });

      const response = await fetch(`/api/threat-events?${params.toString()}`, {
        headers: {
          'Authorization': `Bearer ${sessionToken}`,
          'Content-Type': 'application/json',
        },
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
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchThreatEvents();
  }, [currentPage, searchFilters]);

  const handleFilterChange = (field, value) => {
    setSearchFilters(prev => ({
      ...prev,
      [field]: value
    }));
    setCurrentPage(1);
  };

  const resetFilters = () => {
    setSearchFilters({
      status: '',
      severity: '',
      event_type: '',
      creation_method: '',
      date_from: '',
      date_to: ''
    });
    setCurrentPage(1);
  };

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

  const getSeverityColor = (severity) => {
    switch (severity) {
      case 1: return '#dc2626'; // Critical - Red
      case 2: return '#ea580c'; // High - Orange
      case 3: return '#ca8a04'; // Medium - Yellow
      case 4: return '#16a34a'; // Low - Green
      case 5: return '#6b7280'; // Info - Gray
      default: return '#6b7280'; // Unknown - Gray
    }
  };

  const getSeverityText = (severity) => {
    switch (severity) {
      case 1: return '严重';
      case 2: return '高';
      case 3: return '中';
      case 4: return '低';
      case 5: return '信息';
      default: return '未知';
    }
  };

  const getStatusColor = (status) => {
    switch (status) {
      case 'open': return '#ea580c';
      case 'investigating': return '#ca8a04';
      case 'resolved': return '#16a34a';
      case 'false_positive': return '#6b7280';
      default: return '#6b7280';
    }
  };

  const getStatusText = (status) => {
    switch (status) {
      case 'open': return '开放';
      case 'investigating': return '调查中';
      case 'resolved': return '已解决';
      case 'false_positive': return '误报';
      default: return '未知';
    }
  };

  // 创建威胁事件
  const handleCreateEvent = async () => {
    try {
      const response = await fetch('/api/threat-events', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${sessionToken}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(newEvent),
      });

      const data = await response.json();
      
      if (data.success) {
        setShowCreateModal(false);
        setNewEvent({
          title: '',
          description: '',
          event_type: '',
          severity: 3,
          threat_category: '',
          event_start_time: '',
          event_end_time: '',
          mitre_tactics: [],
          mitre_techniques: [],
          priority: 'medium',
          tags: []
        });
        fetchThreatEvents();
      } else {
        alert('创建威胁事件失败: ' + data.message);
      }
    } catch (error) {
      console.error('Error creating threat event:', error);
      alert('创建威胁事件时发生错误');
    }
  };

  // 查看威胁事件详情
  const viewEventDetail = async (eventId) => {
    try {
      const response = await fetch(`/api/threat-events/${eventId}`, {
        headers: {
          'Authorization': `Bearer ${sessionToken}`,
          'Content-Type': 'application/json',
        },
      });
      const data = await response.json();
      
      if (data.success) {
        setSelectedEvent(data.threat_event);
        setShowDetailModal(true);
      } else {
        alert('获取威胁事件详情失败');
      }
    } catch (error) {
      console.error('Error fetching threat event detail:', error);
      alert('获取威胁事件详情时发生错误');
    }
  };

  const renderCreateModal = () => {
    if (!showCreateModal) return null;

    return (
      <div className="modal-overlay" onClick={() => setShowCreateModal(false)}>
        <div className="modal-content threat-event-create-modal" onClick={(e) => e.stopPropagation()}>
          <div className="modal-header">
            <h3>创建威胁事件</h3>
            <button className="close-button" onClick={() => setShowCreateModal(false)}>
              ✕
            </button>
          </div>
          
          <div className="modal-body">
            <form className="threat-event-form">
              <div className="form-group">
                <label>事件标题 *:</label>
                <input
                  type="text"
                  className="form-input"
                  value={newEvent.title}
                  onChange={(e) => setNewEvent({...newEvent, title: e.target.value})}
                  placeholder="请输入威胁事件标题..."
                />
              </div>

              <div className="form-group">
                <label>事件描述:</label>
                <textarea
                  className="form-textarea"
                  value={newEvent.description}
                  onChange={(e) => setNewEvent({...newEvent, description: e.target.value})}
                  placeholder="详细描述威胁事件..."
                  rows="3"
                />
              </div>

              <div className="form-row">
                <div className="form-group">
                  <label>事件类型 *:</label>
                  <select
                    className="form-select"
                    value={newEvent.event_type}
                    onChange={(e) => setNewEvent({...newEvent, event_type: e.target.value})}
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
                    value={newEvent.severity}
                    onChange={(e) => setNewEvent({...newEvent, severity: parseInt(e.target.value)})}
                  >
                    <option value={1}>严重</option>
                    <option value={2}>高</option>
                    <option value={3}>中</option>
                    <option value={4}>低</option>
                    <option value={5}>信息</option>
                  </select>
                </div>
              </div>

              <div className="form-group">
                <label>威胁类别:</label>
                <input
                  type="text"
                  className="form-input"
                  value={newEvent.threat_category}
                  onChange={(e) => setNewEvent({...newEvent, threat_category: e.target.value})}
                  placeholder="例: APT, ransomware, trojan..."
                />
              </div>

              <div className="form-row">
                <div className="form-group">
                  <label>事件开始时间:</label>
                  <input
                    type="datetime-local"
                    className="form-input"
                    value={newEvent.event_start_time}
                    onChange={(e) => setNewEvent({...newEvent, event_start_time: e.target.value})}
                  />
                </div>

                <div className="form-group">
                  <label>事件结束时间:</label>
                  <input
                    type="datetime-local"
                    className="form-input"
                    value={newEvent.event_end_time}
                    onChange={(e) => setNewEvent({...newEvent, event_end_time: e.target.value})}
                  />
                </div>
              </div>

              <div className="form-group">
                <label>MITRE 技术:</label>
                <input
                  type="text"
                  className="form-input"
                  placeholder="例: T1566,T1059 (逗号分隔)"
                  onChange={(e) => {
                    const techniques = e.target.value.split(',').map(t => t.trim()).filter(t => t);
                    setNewEvent({...newEvent, mitre_techniques: techniques});
                  }}
                />
              </div>

              <div className="form-group">
                <label>优先级:</label>
                <select
                  className="form-select"
                  value={newEvent.priority}
                  onChange={(e) => setNewEvent({...newEvent, priority: e.target.value})}
                >
                  <option value="critical">紧急</option>
                  <option value="high">高</option>
                  <option value="medium">中</option>
                  <option value="low">低</option>
                </select>
              </div>

              <div className="form-group">
                <label>标签:</label>
                <input
                  type="text"
                  className="form-input"
                  placeholder="例: malware,persistence (逗号分隔)"
                  onChange={(e) => {
                    const tags = e.target.value.split(',').map(t => t.trim()).filter(t => t);
                    setNewEvent({...newEvent, tags: tags});
                  }}
                />
              </div>
            </form>
          </div>

          <div className="modal-footer">
            <button className="cancel-button" onClick={() => setShowCreateModal(false)}>
              取消
            </button>
            <button 
              className="save-button" 
              onClick={handleCreateEvent}
              disabled={!newEvent.title || !newEvent.event_type}
            >
              创建事件
            </button>
          </div>
        </div>
      </div>
    );
  };

  const renderDetailModal = () => {
    if (!showDetailModal || !selectedEvent) return null;

    return (
      <div className="modal-overlay" onClick={() => setShowDetailModal(false)}>
        <div className="modal-content threat-event-detail-modal" onClick={(e) => e.stopPropagation()}>
          <div className="modal-header">
            <h3>威胁事件详情</h3>
            <button className="close-button" onClick={() => setShowDetailModal(false)}>
              ✕
            </button>
          </div>
          
          <div className="modal-body">
            <div className="event-detail-section">
              <h4>基本信息</h4>
              <div className="detail-grid">
                <div className="detail-item">
                  <label>标题:</label>
                  <span>{selectedEvent.title}</span>
                </div>
                <div className="detail-item">
                  <label>事件类型:</label>
                  <span>{selectedEvent.event_type}</span>
                </div>
                <div className="detail-item">
                  <label>严重程度:</label>
                  <span 
                    className="severity-badge"
                    style={{ backgroundColor: getSeverityColor(selectedEvent.severity) }}
                  >
                    {getSeverityText(selectedEvent.severity)}
                  </span>
                </div>
                <div className="detail-item">
                  <label>状态:</label>
                  <span 
                    className="status-badge"
                    style={{ backgroundColor: getStatusColor(selectedEvent.status) }}
                  >
                    {getStatusText(selectedEvent.status)}
                  </span>
                </div>
                <div className="detail-item">
                  <label>优先级:</label>
                  <span className={`priority-badge ${selectedEvent.priority}`}>
                    {selectedEvent.priority}
                  </span>
                </div>
                <div className="detail-item">
                  <label>创建方式:</label>
                  <span>{selectedEvent.creation_method === 'manual' ? '手动' : '自动'}</span>
                </div>
              </div>
            </div>

            {selectedEvent.description && (
              <div className="event-detail-section">
                <h4>事件描述</h4>
                <p>{selectedEvent.description}</p>
              </div>
            )}

            {selectedEvent.mitre_techniques && selectedEvent.mitre_techniques.length > 0 && (
              <div className="event-detail-section">
                <h4>MITRE 技术</h4>
                <div className="mitre-techniques">
                  {selectedEvent.mitre_techniques.map((technique, index) => (
                    <span key={index} className="mitre-badge">{technique}</span>
                  ))}
                </div>
              </div>
            )}

            {selectedEvent.associated_alerts && selectedEvent.associated_alerts.length > 0 && (
              <div className="event-detail-section">
                <h4>关联告警 ({selectedEvent.associated_alerts.length})</h4>
                <div className="associated-alerts">
                  {selectedEvent.associated_alerts.map((alert, index) => (
                    <div key={index} className="alert-item">
                      <span className="alert-id">{alert.alert_id}</span>
                      <span className="alert-type">{alert.alert_type}</span>
                      <span className="alert-device">{alert.device_name}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>

          <div className="modal-footer">
            <button className="close-button" onClick={() => setShowDetailModal(false)}>
              关闭
            </button>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div className="threat-event-list-container">
      <div className="threat-event-list-header">
        <h3>威胁事件列表</h3>
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
              disabled={threatEvents.length < itemsPerPage}
              className="pagination-button"
            >
              下一页
            </button>
          </div>
          <button 
            className="create-button" 
            onClick={() => setShowCreateModal(true)}
          >
            ➕ 创建事件
          </button>
          <button onClick={fetchThreatEvents} className="refresh-button" disabled={loading}>
            {loading ? '⏳ 加载中...' : '🔄 刷新'}
          </button>
        </div>
      </div>

      {/* 搜索筛选器 */}
      <div className="search-filters-container">
        <div className="search-filters-header">
          <h3>筛选条件</h3>
          <button onClick={resetFilters} className="reset-filters-button">
            🔄 重置筛选
          </button>
        </div>
        
        <div className="search-filters-grid">
          <div className="filter-group">
            <label>状态:</label>
            <select
              value={searchFilters.status}
              onChange={(e) => handleFilterChange('status', e.target.value)}
              className="search-select"
            >
              <option value="">全部</option>
              <option value="open">开放</option>
              <option value="investigating">调查中</option>
              <option value="resolved">已解决</option>
              <option value="false_positive">误报</option>
            </select>
          </div>

          <div className="filter-group">
            <label>严重程度:</label>
            <select
              value={searchFilters.severity}
              onChange={(e) => handleFilterChange('severity', e.target.value)}
              className="search-select"
            >
              <option value="">全部</option>
              <option value="1">严重</option>
              <option value="2">高</option>
              <option value="3">中</option>
              <option value="4">低</option>
              <option value="5">信息</option>
            </select>
          </div>

          <div className="filter-group">
            <label>事件类型:</label>
            <select
              value={searchFilters.event_type}
              onChange={(e) => handleFilterChange('event_type', e.target.value)}
              className="search-select"
            >
              <option value="">全部</option>
              <option value="malware">恶意软件</option>
              <option value="persistence">持久化</option>
              <option value="lateral_movement">横向移动</option>
              <option value="data_exfiltration">数据泄露</option>
              <option value="command_and_control">命令控制</option>
            </select>
          </div>

          <div className="filter-group">
            <label>创建方式:</label>
            <select
              value={searchFilters.creation_method}
              onChange={(e) => handleFilterChange('creation_method', e.target.value)}
              className="search-select"
            >
              <option value="">全部</option>
              <option value="manual">手动</option>
              <option value="auto">自动</option>
            </select>
          </div>

          <div className="filter-group">
            <label>开始日期:</label>
            <input
              type="datetime-local"
              value={searchFilters.date_from}
              onChange={(e) => handleFilterChange('date_from', e.target.value)}
              className="search-input"
            />
          </div>

          <div className="filter-group">
            <label>结束日期:</label>
            <input
              type="datetime-local"
              value={searchFilters.date_to}
              onChange={(e) => handleFilterChange('date_to', e.target.value)}
              className="search-input"
            />
          </div>
        </div>
      </div>

      {loading ? (
        <div className="loading-indicator">
          <div className="spinner"></div>
          <p>正在加载威胁事件...</p>
        </div>
      ) : (
        <div className="threat-events-table-container">
          <table className="threat-events-table">
            <thead>
              <tr>
                <th>创建时间</th>
                <th>标题</th>
                <th>事件类型</th>
                <th>严重程度</th>
                <th>状态</th>
                <th>优先级</th>
                <th>创建方式</th>
                <th>创建人</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              {threatEvents.length === 0 ? (
                <tr>
                  <td colSpan="9" className="no-data">
                    暂无威胁事件数据
                  </td>
                </tr>
              ) : (
                threatEvents.map((event, index) => (
                  <tr key={event.id || index} className="threat-event-row">
                    <td className="timestamp-cell">
                      {formatTimestamp(event.created_at)}
                    </td>
                    <td className="title-cell" title={event.title}>
                      {event.title.length > 30 ? event.title.substring(0, 30) + '...' : event.title}
                    </td>
                    <td className="event-type-cell">{event.event_type}</td>
                    <td>
                      <span 
                        className="severity-badge"
                        style={{ backgroundColor: getSeverityColor(event.severity) }}
                      >
                        {getSeverityText(event.severity)}
                      </span>
                    </td>
                    <td>
                      <span 
                        className="status-badge"
                        style={{ backgroundColor: getStatusColor(event.status) }}
                      >
                        {getStatusText(event.status)}
                      </span>
                    </td>
                    <td className={`priority-cell priority-${event.priority}`}>
                      {event.priority}
                    </td>
                    <td className="creation-method-cell">
                      {event.creation_method === 'manual' ? '手动' : '自动'}
                    </td>
                    <td className="creator-cell">{event.created_by_username || '-'}</td>
                    <td className="action-cell">
                      <button
                        className="view-button"
                        onClick={() => viewEventDetail(event.id)}
                      >
                        👁️ 查看
                      </button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      )}

      {renderCreateModal()}
      {renderDetailModal()}
    </div>
  );
};

export default ThreatEventList;