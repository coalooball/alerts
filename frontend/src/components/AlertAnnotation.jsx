import React, { useState, useEffect } from 'react';
import '../App.css';

const AlertAnnotation = () => {
  const [activeTab, setActiveTab] = useState('alert-data');
  const [alertData, setAlertData] = useState([]);
  const [annotations, setAnnotations] = useState([]);
  const [loading, setLoading] = useState(false);
  const [selectedAlert, setSelectedAlert] = useState(null);
  const [showAnnotationModal, setShowAnnotationModal] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const [itemsPerPage] = useState(20);
  const [searchFilters, setSearchFilters] = useState({
    processing_status: 'unprocessed',
    alert_type: '',
    severity: '',
    device_name: '',
    date_from: '',
    date_to: ''
  });

  // 获取待标注的告警数据
  const fetchAlertData = async () => {
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

      const response = await fetch(`/api/alert-data?${params.toString()}`);
      const data = await response.json();
      
      if (data.success) {
        setAlertData(data.alert_data);
      } else {
        console.error('Failed to fetch alert data');
        setAlertData([]);
      }
    } catch (error) {
      console.error('Error fetching alert data:', error);
      setAlertData([]);
    } finally {
      setLoading(false);
    }
  };

  // 获取标注数据
  const fetchAnnotations = async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams();
      params.append('limit', itemsPerPage.toString());
      params.append('offset', ((currentPage - 1) * itemsPerPage).toString());

      const response = await fetch(`/api/annotations?${params.toString()}`);
      const data = await response.json();
      
      if (data.success) {
        setAnnotations(data.annotations);
      } else {
        console.error('Failed to fetch annotations');
        setAnnotations([]);
      }
    } catch (error) {
      console.error('Error fetching annotations:', error);
      setAnnotations([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (activeTab === 'alert-data') {
      fetchAlertData();
    } else if (activeTab === 'annotations') {
      fetchAnnotations();
    }
  }, [activeTab, currentPage, searchFilters]);

  const handleFilterChange = (field, value) => {
    setSearchFilters(prev => ({
      ...prev,
      [field]: value
    }));
    setCurrentPage(1);
  };

  const resetFilters = () => {
    setSearchFilters({
      processing_status: 'unprocessed',
      alert_type: '',
      severity: '',
      device_name: '',
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
      default: return '#6b7280'; // Unknown - Gray
    }
  };

  const getSeverityText = (severity) => {
    switch (severity) {
      case 1: return '严重';
      case 2: return '高';
      case 3: return '中';
      case 4: return '低';
      default: return '未知';
    }
  };

  const openAnnotationModal = (alert) => {
    setSelectedAlert(alert);
    setShowAnnotationModal(true);
  };

  const closeAnnotationModal = () => {
    setSelectedAlert(null);
    setShowAnnotationModal(false);
  };

  const renderAlertDataTab = () => (
    <div className="alert-annotation-content">
      <div className="alert-annotation-header">
        <h3>待标注告警数据</h3>
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
              disabled={alertData.length < itemsPerPage}
              className="pagination-button"
            >
              下一页
            </button>
          </div>
          <button onClick={fetchAlertData} className="refresh-button" disabled={loading}>
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
            <label>处理状态:</label>
            <select
              value={searchFilters.processing_status}
              onChange={(e) => handleFilterChange('processing_status', e.target.value)}
              className="search-select"
            >
              <option value="">全部</option>
              <option value="unprocessed">未处理</option>
              <option value="processed">已处理</option>
              <option value="ignored">已忽略</option>
            </select>
          </div>

          <div className="filter-group">
            <label>告警类型:</label>
            <input
              type="text"
              placeholder="搜索告警类型..."
              value={searchFilters.alert_type}
              onChange={(e) => handleFilterChange('alert_type', e.target.value)}
              className="search-input"
            />
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
            </select>
          </div>

          <div className="filter-group">
            <label>设备名称:</label>
            <input
              type="text"
              placeholder="搜索设备名称..."
              value={searchFilters.device_name}
              onChange={(e) => handleFilterChange('device_name', e.target.value)}
              className="search-input"
            />
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
          <p>正在加载告警数据...</p>
        </div>
      ) : (
        <div className="alerts-table-container">
          <table className="alerts-table">
            <thead>
              <tr>
                <th>时间</th>
                <th>告警类型</th>
                <th>严重程度</th>
                <th>设备名称</th>
                <th>设备IP</th>
                <th>处理状态</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              {alertData.length === 0 ? (
                <tr>
                  <td colSpan="7" className="no-data">
                    暂无告警数据
                  </td>
                </tr>
              ) : (
                alertData.map((alert, index) => (
                  <tr key={alert.id || index} className="alert-row">
                    <td className="timestamp-cell">
                      {formatTimestamp(alert.alert_timestamp)}
                    </td>
                    <td className="alert-type-cell">{alert.alert_type}</td>
                    <td>
                      <span 
                        className="severity-badge"
                        style={{ backgroundColor: getSeverityColor(alert.severity) }}
                      >
                        {getSeverityText(alert.severity)}
                      </span>
                    </td>
                    <td className="device-name-cell">{alert.device_name || '-'}</td>
                    <td className="ip-cell">{alert.device_ip || '-'}</td>
                    <td className="status-cell">
                      <span className={`status-badge status-${alert.processing_status}`}>
                        {alert.processing_status === 'unprocessed' ? '未处理' : 
                         alert.processing_status === 'processed' ? '已处理' : '已忽略'}
                      </span>
                    </td>
                    <td className="action-cell">
                      <button
                        className="annotation-button"
                        onClick={() => openAnnotationModal(alert)}
                        disabled={alert.processing_status === 'processed'}
                      >
                        📝 标注
                      </button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );

  const renderAnnotationsTab = () => (
    <div className="annotations-content">
      <div className="annotations-header">
        <h3>标注记录</h3>
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
              disabled={annotations.length < itemsPerPage}
              className="pagination-button"
            >
              下一页
            </button>
          </div>
          <button onClick={fetchAnnotations} className="refresh-button" disabled={loading}>
            {loading ? '⏳ 加载中...' : '🔄 刷新'}
          </button>
        </div>
      </div>

      {loading ? (
        <div className="loading-indicator">
          <div className="spinner"></div>
          <p>正在加载标注记录...</p>
        </div>
      ) : (
        <div className="annotations-table-container">
          <table className="annotations-table">
            <thead>
              <tr>
                <th>标注时间</th>
                <th>标注类型</th>
                <th>威胁等级</th>
                <th>恶意性</th>
                <th>标注人</th>
                <th>审核状态</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              {annotations.length === 0 ? (
                <tr>
                  <td colSpan="7" className="no-data">
                    暂无标注记录
                  </td>
                </tr>
              ) : (
                annotations.map((annotation, index) => (
                  <tr key={annotation.id || index} className="annotation-row">
                    <td className="timestamp-cell">
                      {formatTimestamp(annotation.annotated_at)}
                    </td>
                    <td className="type-cell">{annotation.annotation_type}</td>
                    <td className="threat-level-cell">
                      <span className={`threat-level-badge ${annotation.threat_level}`}>
                        {annotation.threat_level || '-'}
                      </span>
                    </td>
                    <td className="malicious-cell">
                      {annotation.is_malicious === null ? '-' : 
                       annotation.is_malicious ? '✅ 恶意' : '❌ 良性'}
                    </td>
                    <td className="annotator-cell">{annotation.annotated_by_username}</td>
                    <td className="review-status-cell">
                      <span className={`review-status-badge ${annotation.review_status}`}>
                        {annotation.review_status === 'pending' ? '待审核' :
                         annotation.review_status === 'approved' ? '已通过' : '已拒绝'}
                      </span>
                    </td>
                    <td className="action-cell">
                      <button className="view-button">👁️ 查看</button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );

  const renderAnnotationModal = () => {
    if (!showAnnotationModal || !selectedAlert) return null;

    return (
      <div className="modal-overlay" onClick={closeAnnotationModal}>
        <div className="modal-content annotation-modal" onClick={(e) => e.stopPropagation()}>
          <div className="modal-header">
            <h3>告警标注</h3>
            <button className="close-button" onClick={closeAnnotationModal}>
              ✕
            </button>
          </div>
          
          <div className="modal-body">
            <div className="alert-info-section">
              <h4>告警信息</h4>
              <div className="alert-info-grid">
                <div className="info-item">
                  <label>告警类型:</label>
                  <span>{selectedAlert.alert_type}</span>
                </div>
                <div className="info-item">
                  <label>设备名称:</label>
                  <span>{selectedAlert.device_name}</span>
                </div>
                <div className="info-item">
                  <label>严重程度:</label>
                  <span 
                    className="severity-badge"
                    style={{ backgroundColor: getSeverityColor(selectedAlert.severity) }}
                  >
                    {getSeverityText(selectedAlert.severity)}
                  </span>
                </div>
                <div className="info-item">
                  <label>时间:</label>
                  <span>{formatTimestamp(selectedAlert.alert_timestamp)}</span>
                </div>
              </div>
            </div>

            <div className="annotation-form-section">
              <h4>标注信息</h4>
              <form className="annotation-form">
                <div className="form-group">
                  <label>标注类型:</label>
                  <select className="form-select">
                    <option value="">请选择...</option>
                    <option value="threat_indicator">威胁指标</option>
                    <option value="false_positive">误报</option>
                    <option value="benign">良性</option>
                    <option value="malicious">恶意</option>
                  </select>
                </div>

                <div className="form-group">
                  <label>威胁等级:</label>
                  <select className="form-select">
                    <option value="">请选择...</option>
                    <option value="critical">严重</option>
                    <option value="high">高</option>
                    <option value="medium">中</option>
                    <option value="low">低</option>
                    <option value="info">信息</option>
                  </select>
                </div>

                <div className="form-group">
                  <label>恶意性判断:</label>
                  <div className="radio-group">
                    <label><input type="radio" name="malicious" value="true" /> 恶意</label>
                    <label><input type="radio" name="malicious" value="false" /> 良性</label>
                    <label><input type="radio" name="malicious" value="" defaultChecked /> 未知</label>
                  </div>
                </div>

                <div className="form-group">
                  <label>MITRE 技术:</label>
                  <input type="text" className="form-input" placeholder="例: T1566, T1059" />
                </div>

                <div className="form-group">
                  <label>攻击阶段:</label>
                  <input type="text" className="form-input" placeholder="例: initial_access, execution" />
                </div>

                <div className="form-group">
                  <label>标注标题:</label>
                  <input type="text" className="form-input" placeholder="简要描述..." />
                </div>

                <div className="form-group">
                  <label>详细描述:</label>
                  <textarea className="form-textarea" placeholder="详细的威胁分析描述..." rows="4"></textarea>
                </div>

                <div className="form-group">
                  <label>备注:</label>
                  <textarea className="form-textarea" placeholder="其他备注信息..." rows="2"></textarea>
                </div>
              </form>
            </div>
          </div>

          <div className="modal-footer">
            <button className="cancel-button" onClick={closeAnnotationModal}>
              取消
            </button>
            <button className="save-button">
              保存标注
            </button>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div className="alert-annotation-container">
      <div className="alert-annotation-main-header">
        <h2>📝 告警数据标注</h2>
        <div className="alert-annotation-tabs">
          <button 
            className={`tab-button ${activeTab === 'alert-data' ? 'active' : ''}`}
            onClick={() => setActiveTab('alert-data')}
          >
            📋 待标注数据
          </button>
          <button 
            className={`tab-button ${activeTab === 'annotations' ? 'active' : ''}`}
            onClick={() => setActiveTab('annotations')}
          >
            📝 标注记录
          </button>
        </div>
      </div>

      {activeTab === 'alert-data' && renderAlertDataTab()}
      {activeTab === 'annotations' && renderAnnotationsTab()}
      
      {renderAnnotationModal()}
    </div>
  );
};

export default AlertAnnotation;