import React, { useState, useEffect } from 'react';
import '../App.css';
import AlertAnalysis from './AlertAnalysis';

const AlertData = () => {
  const [activeTab, setActiveTab] = useState('list');
  const [alerts, setAlerts] = useState([]);
  const [filteredAlerts, setFilteredAlerts] = useState([]);
  const [annotations, setAnnotations] = useState([]);
  const [loading, setLoading] = useState(false);
  const [selectedAlert, setSelectedAlert] = useState(null);
  const [showDetail, setShowDetail] = useState(false);
  const [showAnnotationModal, setShowAnnotationModal] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const [alertsPerPage] = useState(20);
  const [totalAlerts, setTotalAlerts] = useState(0);
  const [refreshInterval, setRefreshInterval] = useState(() => {
    // Load saved refresh interval from localStorage
    const saved = localStorage.getItem('alertRefreshInterval');
    return saved ? parseInt(saved, 10) : 0;
  });
  const [autoRefreshTimer, setAutoRefreshTimer] = useState(null);
  const [searchFilters, setSearchFilters] = useState({
    deviceName: '',
    deviceIp: '',
    alertType: '',
    threatCategory: '',
    severity: '',
    dataType: '',
    kafkaSource: '',
    dateFrom: '',
    dateTo: ''
  });

  // Initial load
  useEffect(() => {
    fetchAlerts();
  }, []);

  // Save refresh interval to localStorage when it changes
  useEffect(() => {
    localStorage.setItem('alertRefreshInterval', refreshInterval.toString());
  }, [refreshInterval]);

  useEffect(() => {
    if (refreshInterval > 0) {
      const timer = setInterval(() => {
        fetchAlerts();
      }, refreshInterval * 1000);
      setAutoRefreshTimer(timer);
      return () => {
        clearInterval(timer);
      };
    } else {
      if (autoRefreshTimer) {
        clearInterval(autoRefreshTimer);
        setAutoRefreshTimer(null);
      }
    }
  }, [refreshInterval, currentPage]);

  const fetchAlerts = async () => {
    setLoading(true);
    try {
      // Build query parameters for backend filtering
      const params = new URLSearchParams();
      params.append('limit', alertsPerPage.toString());
      params.append('offset', ((currentPage - 1) * alertsPerPage).toString());
      
      // Add search filters to query parameters
      if (searchFilters.deviceName) {
        params.append('device_name', searchFilters.deviceName);
      }
      if (searchFilters.deviceIp) {
        params.append('device_ip', searchFilters.deviceIp);
      }
      if (searchFilters.alertType) {
        params.append('alert_type', searchFilters.alertType);
      }
      if (searchFilters.threatCategory) {
        params.append('threat_category', searchFilters.threatCategory);
      }
      if (searchFilters.severity) {
        params.append('severity', searchFilters.severity);
      }
      if (searchFilters.dataType) {
        params.append('data_type', searchFilters.dataType);
      }
      if (searchFilters.kafkaSource) {
        params.append('kafka_source', searchFilters.kafkaSource);
      }
      if (searchFilters.dateFrom) {
        params.append('date_from', searchFilters.dateFrom);
      }
      if (searchFilters.dateTo) {
        params.append('date_to', searchFilters.dateTo);
      }
      
      const response = await fetch(`/api/alerts?${params.toString()}`);
      const data = await response.json();
      
      if (data.success) {
        setAlerts(data.alerts);
        setFilteredAlerts(data.alerts);
        setTotalAlerts(data.total || data.alerts.length);
      } else {
        console.error('Failed to fetch alerts');
        setAlerts([]);
        setFilteredAlerts([]);
        setTotalAlerts(0);
      }
    } catch (error) {
      console.error('Error fetching alerts:', error);
      setAlerts([]);
      setFilteredAlerts([]);
      setTotalAlerts(0);
    } finally {
      setLoading(false);
    }
  };

  const fetchAlertDetail = async (alertId) => {
    try {
      const response = await fetch(`/api/alerts/${encodeURIComponent(alertId)}`);
      const data = await response.json();
      
      if (data.success && data.alert) {
        setSelectedAlert(data.alert);
        setShowDetail(true);
      } else {
        console.error('Failed to fetch alert detail');
      }
    } catch (error) {
      console.error('Error fetching alert detail:', error);
    }
  };

  // 获取标注记录
  const fetchAnnotations = async () => {
    setLoading(true);
    try {
      const sessionToken = localStorage.getItem('sessionToken');
      const response = await fetch('/api/annotations', {
        headers: {
          'Authorization': `Bearer ${sessionToken}`
        }
      });
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

  // 打开标注模态框
  const openAnnotationModal = (alert) => {
    setSelectedAlert(alert);
    setAnnotationForm({
      annotation_type: '',
      threat_level: '',
      is_malicious: null,
      mitre_techniques: '',
      attack_stage: '',
      title: '',
      description: '',
      notes: ''
    });
    setShowAnnotationModal(true);
  };

  // 关闭标注模态框
  const closeAnnotationModal = () => {
    setSelectedAlert(null);
    setShowAnnotationModal(false);
    setAnnotationForm({
      annotation_type: '',
      threat_level: '',
      is_malicious: null,
      mitre_techniques: '',
      attack_stage: '',
      title: '',
      description: '',
      notes: ''
    });
  };

  // 保存标注
  const saveAnnotation = async (annotationData) => {
    try {
      const sessionToken = localStorage.getItem('sessionToken');
      const response = await fetch('/api/annotations', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${sessionToken}`
        },
        body: JSON.stringify({
          alert_data_id: selectedAlert.id,
          ...annotationData
        }),
      });

      const data = await response.json();
      
      if (data.success) {
        closeAnnotationModal();
        // 如果当前在标注记录Tab，刷新标注数据
        if (activeTab === 'annotations') {
          fetchAnnotations();
        }
        if (window.showToast) {
          window.showToast('标注保存成功！', 'success');
        }
      } else {
        if (window.showToast) {
          window.showToast('标注保存失败: ' + data.message, 'error');
        }
      }
    } catch (error) {
      console.error('Error saving annotation:', error);
      if (window.showToast) {
        window.showToast('保存标注时发生错误', 'error');
      }
    }
  };

  // Server-side filtering is now handled by the backend
  // No client-side filtering needed

  const resetFilters = () => {
    setSearchFilters({
      deviceName: '',
      deviceIp: '',
      alertType: '',
      threatCategory: '',
      severity: '',
      dataType: '',
      kafkaSource: '',
      dateFrom: '',
      dateTo: ''
    });
    setCurrentPage(1);
  };

  const handleFilterChange = (field, value) => {
    setSearchFilters(prev => ({
      ...prev,
      [field]: value
    }));
  };

  // Fetch new data when search filters change (backend filtering)
  useEffect(() => {
    fetchAlerts();
  }, [searchFilters, currentPage]);

  // 根据activeTab加载数据
  useEffect(() => {
    if (activeTab === 'annotations') {
      fetchAnnotations();
    }
  }, [activeTab]);

  const handleRowClick = (alert) => {
    fetchAlertDetail(alert.id);
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

  const getDataTypeColor = (dataType) => {
    switch (dataType) {
      case 'edr': return '#3b82f6'; // Blue
      case 'ngav': return '#8b5cf6'; // Purple
      default: return '#6b7280'; // Gray
    }
  };

  const closeDetailModal = () => {
    setShowDetail(false);
    setSelectedAlert(null);
  };

  const renderDetailModal = () => {
    if (!showDetail || !selectedAlert) return null;

    return (
      <div className="modal-overlay" onClick={closeDetailModal}>
        <div className="modal-content alert-detail-modal" onClick={(e) => e.stopPropagation()}>
          <div className="modal-header">
            <h3>告警详细信息</h3>
            <button className="close-button" onClick={closeDetailModal}>
              ✕
            </button>
          </div>
          
          <div className="modal-body">
            <div className="alert-detail-grid">
              {/* Basic Information */}
              <div className="detail-section">
                <h4>基础信息</h4>
                <div className="detail-item">
                  <label>告警ID:</label>
                  <span>{selectedAlert.id}</span>
                </div>
                <div className="detail-item">
                  <label>数据类型:</label>
                  <span 
                    className="data-type-badge"
                    style={{ backgroundColor: getDataTypeColor(selectedAlert.data_type) }}
                  >
                    {selectedAlert.data_type?.toUpperCase()}
                  </span>
                </div>
                <div className="detail-item">
                  <label>严重程度:</label>
                  <span 
                    className="severity-badge"
                    style={{ backgroundColor: getSeverityColor(selectedAlert.severity) }}
                  >
                    {getSeverityText(selectedAlert.severity)}
                  </span>
                </div>
                <div className="detail-item">
                  <label>告警类型:</label>
                  <span>{selectedAlert.alert_type}</span>
                </div>
                <div className="detail-item">
                  <label>创建时间:</label>
                  <span>{formatTimestamp(selectedAlert.create_time)}</span>
                </div>
                <div className="detail-item">
                  <label>Kafka来源:</label>
                  <span>{selectedAlert.kafka_config_name || '未知'}</span>
                </div>
              </div>

              {/* Device Information */}
              <div className="detail-section">
                <h4>设备信息</h4>
                <div className="detail-item">
                  <label>设备名称:</label>
                  <span>{selectedAlert.device_name}</span>
                </div>
                <div className="detail-item">
                  <label>设备ID:</label>
                  <span>{selectedAlert.device_id}</span>
                </div>
                <div className="detail-item">
                  <label>操作系统:</label>
                  <span>{selectedAlert.device_os}</span>
                </div>
                <div className="detail-item">
                  <label>内网IP:</label>
                  <span>{selectedAlert.device_internal_ip}</span>
                </div>
                <div className="detail-item">
                  <label>外网IP:</label>
                  <span>{selectedAlert.device_external_ip}</span>
                </div>
                <div className="detail-item">
                  <label>用户名:</label>
                  <span>{selectedAlert.device_username}</span>
                </div>
              </div>

              {/* Threat Information */}
              {selectedAlert.threat_category && (
                <div className="detail-section full-width">
                  <h4>威胁信息</h4>
                  <div className="detail-item">
                    <label>威胁类别:</label>
                    <span>{selectedAlert.threat_category}</span>
                  </div>
                </div>
              )}

              {/* Raw Data */}
              <div className="detail-section full-width">
                <h4>原始数据</h4>
                <div className="raw-data-container">
                  <pre>{JSON.stringify(JSON.parse(selectedAlert.raw_data), null, 2)}</pre>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  };


  // Check if any filters are active
  const hasActiveFilters = searchFilters.deviceName || 
                          searchFilters.deviceIp || 
                          searchFilters.alertType || 
                          searchFilters.threatCategory || 
                          searchFilters.severity || 
                          searchFilters.dataType || 
                          searchFilters.kafkaSource || 
                          searchFilters.dateFrom || 
                          searchFilters.dateTo;

  // Backend handles pagination, so we display the returned alerts directly
  const paginatedAlerts = filteredAlerts;
  const totalPages = Math.ceil(totalAlerts / alertsPerPage);
  const filteredTotal = totalAlerts; // Backend returns total for current filters

  // 渲染标注记录列表
  const renderAnnotationsList = () => (
    <div className="annotations-list-content">
      <div className="annotations-header">
        <h3>标注记录列表</h3>
        <div className="header-controls">
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
                <th>告警ID</th>
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
                  <td colSpan="8" className="no-data">
                    暂无标注记录
                  </td>
                </tr>
              ) : (
                annotations.map((annotation, index) => (
                  <tr key={annotation.id || index} className="annotation-row">
                    <td className="timestamp-cell">
                      {formatTimestamp(annotation.annotated_at)}
                    </td>
                    <td className="alert-id-cell">{annotation.alert_data_id}</td>
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

  // 标注表单状态
  const [annotationForm, setAnnotationForm] = useState({
    annotation_type: '',
    threat_level: '',
    is_malicious: null,
    mitre_techniques: '',
    attack_stage: '',
    title: '',
    description: '',
    notes: ''
  });

  // 渲染标注模态框
  const renderAnnotationModal = () => {
    if (!showAnnotationModal || !selectedAlert) return null;

    const handleFormChange = (field, value) => {
      setAnnotationForm(prev => ({
        ...prev,
        [field]: value
      }));
    };

    const handleSave = () => {
      const formData = {
        ...annotationForm,
        mitre_techniques: annotationForm.mitre_techniques ? 
          annotationForm.mitre_techniques.split(',').map(t => t.trim()).filter(t => t) : [],
        is_malicious: annotationForm.is_malicious === 'true' ? true : 
                     annotationForm.is_malicious === 'false' ? false : null
      };
      saveAnnotation(formData);
    };

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
                  <label>告警ID:</label>
                  <span>{selectedAlert.id}</span>
                </div>
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
              </div>
            </div>

            <div className="annotation-form-section">
              <h4>标注信息</h4>
              <form className="annotation-form">
                <div className="form-group">
                  <label>标注类型:</label>
                  <select 
                    className="form-select"
                    value={annotationForm.annotation_type}
                    onChange={(e) => handleFormChange('annotation_type', e.target.value)}
                  >
                    <option value="">请选择...</option>
                    <option value="threat_indicator">威胁指标</option>
                    <option value="false_positive">误报</option>
                    <option value="benign">良性</option>
                    <option value="malicious">恶意</option>
                  </select>
                </div>

                <div className="form-group">
                  <label>威胁等级:</label>
                  <select 
                    className="form-select"
                    value={annotationForm.threat_level}
                    onChange={(e) => handleFormChange('threat_level', e.target.value)}
                  >
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
                    <label>
                      <input 
                        type="radio" 
                        name="malicious" 
                        value="true" 
                        checked={annotationForm.is_malicious === 'true'}
                        onChange={(e) => handleFormChange('is_malicious', e.target.value)}
                      /> 恶意
                    </label>
                    <label>
                      <input 
                        type="radio" 
                        name="malicious" 
                        value="false" 
                        checked={annotationForm.is_malicious === 'false'}
                        onChange={(e) => handleFormChange('is_malicious', e.target.value)}
                      /> 良性
                    </label>
                    <label>
                      <input 
                        type="radio" 
                        name="malicious" 
                        value="" 
                        checked={annotationForm.is_malicious === '' || annotationForm.is_malicious === null}
                        onChange={(e) => handleFormChange('is_malicious', e.target.value)}
                      /> 未知
                    </label>
                  </div>
                </div>

                <div className="form-group">
                  <label>MITRE 技术:</label>
                  <input 
                    type="text" 
                    className="form-input" 
                    placeholder="例: T1566, T1059 (逗号分隔)"
                    value={annotationForm.mitre_techniques}
                    onChange={(e) => handleFormChange('mitre_techniques', e.target.value)}
                  />
                </div>

                <div className="form-group">
                  <label>攻击阶段:</label>
                  <input 
                    type="text" 
                    className="form-input" 
                    placeholder="例: initial_access, execution"
                    value={annotationForm.attack_stage}
                    onChange={(e) => handleFormChange('attack_stage', e.target.value)}
                  />
                </div>

                <div className="form-group">
                  <label>标注标题:</label>
                  <input 
                    type="text" 
                    className="form-input" 
                    placeholder="简要描述..."
                    value={annotationForm.title}
                    onChange={(e) => handleFormChange('title', e.target.value)}
                  />
                </div>

                <div className="form-group">
                  <label>详细描述:</label>
                  <textarea 
                    className="form-textarea" 
                    placeholder="详细的威胁分析描述..." 
                    rows="4"
                    value={annotationForm.description}
                    onChange={(e) => handleFormChange('description', e.target.value)}
                  />
                </div>

                <div className="form-group">
                  <label>备注:</label>
                  <textarea 
                    className="form-textarea" 
                    placeholder="其他备注信息..." 
                    rows="2"
                    value={annotationForm.notes}
                    onChange={(e) => handleFormChange('notes', e.target.value)}
                  />
                </div>
              </form>
            </div>
          </div>

          <div className="modal-footer">
            <button className="cancel-button" onClick={closeAnnotationModal}>
              取消
            </button>
            <button 
              className="save-button" 
              onClick={handleSave}
              disabled={!annotationForm.annotation_type}
            >
              保存标注
            </button>
          </div>
        </div>
      </div>
    );
  };

  const renderAlertList = () => (
    <div className="alert-list-content">
      <div className="alert-data-header">
        <h3>告警列表 
          <span className="total-alerts-count">
            (总数: {totalAlerts.toLocaleString('zh-CN')}
            {hasActiveFilters && filteredTotal !== totalAlerts && (
              <span>, 筛选后: {filteredTotal.toLocaleString('zh-CN')}</span>
            )})
          </span>
        </h3>
        <div className="header-controls">
          <div className="pagination">
            <button 
              onClick={() => setCurrentPage(prev => Math.max(prev - 1, 1))}
              disabled={currentPage === 1}
              className="pagination-button"
            >
              上一页
            </button>
            <span className="page-info">第 {currentPage} 页 / 共 {totalPages} 页</span>
            <button 
              onClick={() => setCurrentPage(prev => prev + 1)}
              disabled={currentPage >= totalPages}
              className="pagination-button"
            >
              下一页
            </button>
          </div>
          <select 
            className="refresh-interval-select"
            value={refreshInterval}
            onChange={(e) => {
              const newInterval = Number(e.target.value);
              setRefreshInterval(newInterval);
            }}
          >
            <option value={0}>手动刷新</option>
            <option value={5}>每 5 秒</option>
            <option value={10}>每 10 秒</option>
            <option value={30}>每 30 秒</option>
            <option value={60}>每 1 分钟</option>
            <option value={300}>每 5 分钟</option>
          </select>
          <button onClick={fetchAlerts} className="refresh-button" disabled={loading}>
            {loading ? '⏳ 加载中...' : '🔄 刷新'}
          </button>
        </div>
      </div>

      {/* Search Filters */}
      <div className="search-filters-container">
        <div className="search-filters-header">
          <h3>搜索筛选</h3>
          <button onClick={resetFilters} className="reset-filters-button">
            🔄 重置筛选
          </button>
        </div>
        
        <div className="search-filters-grid">
          <div className="filter-group">
            <label>设备名称:</label>
            <input
              type="text"
              placeholder="搜索设备名称..."
              value={searchFilters.deviceName}
              onChange={(e) => handleFilterChange('deviceName', e.target.value)}
              className="search-input"
            />
          </div>

          <div className="filter-group">
            <label>设备IP:</label>
            <input
              type="text"
              placeholder="搜索IP地址..."
              value={searchFilters.deviceIp}
              onChange={(e) => handleFilterChange('deviceIp', e.target.value)}
              className="search-input"
            />
          </div>

          <div className="filter-group">
            <label>告警类型:</label>
            <input
              type="text"
              placeholder="搜索告警类型..."
              value={searchFilters.alertType}
              onChange={(e) => handleFilterChange('alertType', e.target.value)}
              className="search-input"
            />
          </div>

          <div className="filter-group">
            <label>威胁类别:</label>
            <input
              type="text"
              placeholder="搜索威胁类别..."
              value={searchFilters.threatCategory}
              onChange={(e) => handleFilterChange('threatCategory', e.target.value)}
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
            <label>数据类型:</label>
            <select
              value={searchFilters.dataType}
              onChange={(e) => handleFilterChange('dataType', e.target.value)}
              className="search-select"
            >
              <option value="">全部</option>
              <option value="edr">EDR</option>
              <option value="ngav">NGAV</option>
            </select>
          </div>

          <div className="filter-group">
            <label>Kafka来源:</label>
            <input
              type="text"
              placeholder="搜索Kafka来源..."
              value={searchFilters.kafkaSource}
              onChange={(e) => handleFilterChange('kafkaSource', e.target.value)}
              className="search-input"
            />
          </div>

          <div className="filter-group">
            <label>开始日期:</label>
            <input
              type="datetime-local"
              value={searchFilters.dateFrom}
              onChange={(e) => handleFilterChange('dateFrom', e.target.value)}
              className="search-input"
            />
          </div>

          <div className="filter-group">
            <label>结束日期:</label>
            <input
              type="datetime-local"
              value={searchFilters.dateTo}
              onChange={(e) => handleFilterChange('dateTo', e.target.value)}
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
        <>
          <div className="alerts-table-container">
            <table className="alerts-table">
              <thead>
                <tr>
                  <th>时间</th>
                  <th>类型</th>
                  <th>严重程度</th>
                  <th>设备名称</th>
                  <th>设备IP</th>
                  <th>告警类型</th>
                  <th>威胁类别</th>
                  <th>Kafka来源</th>
                  <th>操作</th>
                </tr>
              </thead>
              <tbody>
                {paginatedAlerts.length === 0 ? (
                  <tr>
                    <td colSpan="9" className="no-data">
                      {hasActiveFilters && filteredTotal === 0 && totalAlerts > 0 ? '没有符合筛选条件的告警数据' : '暂无告警数据'}
                    </td>
                  </tr>
                ) : (
                  paginatedAlerts.map((alert, index) => (
                    <tr 
                      key={alert.id || index} 
                      className="alert-row"
                    >
                      <td className="timestamp-cell">
                        {formatTimestamp(alert.create_time)}
                      </td>
                      <td>
                        <span 
                          className="data-type-badge"
                          style={{ backgroundColor: getDataTypeColor(alert.data_type) }}
                        >
                          {alert.data_type?.toUpperCase()}
                        </span>
                      </td>
                      <td>
                        <span 
                          className="severity-badge"
                          style={{ backgroundColor: getSeverityColor(alert.severity) }}
                        >
                          {getSeverityText(alert.severity)}
                        </span>
                      </td>
                      <td className="device-name-cell">{alert.device_name}</td>
                      <td className="ip-cell">{alert.device_internal_ip}</td>
                      <td className="alert-type-cell">{alert.alert_type}</td>
                      <td className="threat-category-cell">
                        {alert.threat_category || '-'}
                      </td>
                      <td className="kafka-source-cell">
                        {alert.kafka_config_name || '-'}
                      </td>
                      <td className="action-cell">
                        <button
                          className="detail-button"
                          onClick={() => handleRowClick(alert)}
                          title="查看详情"
                        >
                          👁️ 详情
                        </button>
                        <button
                          className="annotation-button"
                          onClick={() => openAnnotationModal(alert)}
                          title="添加标注"
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
        </>
      )}

      {renderDetailModal()}
      {renderAnnotationModal()}
    </div>
  );

  return (
    <div className="alert-data-container">
      <div className="alert-data-main-header">
        <h2>🚨 告警数据</h2>
        <div className="alert-data-tabs">
          <button 
            className={`tab-button ${activeTab === 'list' ? 'active' : ''}`}
            onClick={() => setActiveTab('list')}
          >
            📋 告警列表
          </button>
          <button 
            className={`tab-button ${activeTab === 'annotations' ? 'active' : ''}`}
            onClick={() => setActiveTab('annotations')}
          >
            📝 标注记录
          </button>
          <button 
            className={`tab-button ${activeTab === 'analysis' ? 'active' : ''}`}
            onClick={() => setActiveTab('analysis')}
          >
            📊 告警分析
          </button>
        </div>
      </div>

      {activeTab === 'list' && renderAlertList()}
      {activeTab === 'annotations' && renderAnnotationsList()}
      {activeTab === 'analysis' && <AlertAnalysis />}
    </div>
  );
};

export default AlertData;