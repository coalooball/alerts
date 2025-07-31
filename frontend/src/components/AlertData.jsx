import React, { useState, useEffect } from 'react';
import '../App.css';

const AlertData = () => {
  const [alerts, setAlerts] = useState([]);
  const [loading, setLoading] = useState(false);
  const [selectedAlert, setSelectedAlert] = useState(null);
  const [showDetail, setShowDetail] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const [alertsPerPage] = useState(20);

  useEffect(() => {
    fetchAlerts();
  }, [currentPage]);

  const fetchAlerts = async () => {
    setLoading(true);
    try {
      const offset = (currentPage - 1) * alertsPerPage;
      const response = await fetch(`/api/alerts?limit=${alertsPerPage}&offset=${offset}`);
      const data = await response.json();
      
      if (data.success) {
        setAlerts(data.alerts);
      } else {
        console.error('Failed to fetch alerts');
        setAlerts([]);
      }
    } catch (error) {
      console.error('Error fetching alerts:', error);
      setAlerts([]);
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

  const renderPagination = () => {
    return (
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
          disabled={alerts.length < alertsPerPage}
          className="pagination-button"
        >
          下一页
        </button>
      </div>
    );
  };

  return (
    <div className="alert-data-container">
      <div className="alert-data-header">
        <h2>告警数据</h2>
        <button onClick={fetchAlerts} className="refresh-button" disabled={loading}>
          {loading ? '⏳ 加载中...' : '🔄 刷新'}
        </button>
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
                </tr>
              </thead>
              <tbody>
                {alerts.length === 0 ? (
                  <tr>
                    <td colSpan="7" className="no-data">
                      暂无告警数据
                    </td>
                  </tr>
                ) : (
                  alerts.map((alert, index) => (
                    <tr 
                      key={alert.id || index} 
                      className="alert-row"
                      onClick={() => handleRowClick(alert)}
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
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>

          {alerts.length > 0 && renderPagination()}
        </>
      )}

      {renderDetailModal()}
    </div>
  );
};

export default AlertData;