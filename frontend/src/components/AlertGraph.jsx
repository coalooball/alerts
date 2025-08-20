import React, { useEffect, useRef, useState } from 'react';
import { Network } from 'vis-network/standalone';
import { DataSet } from 'vis-data/standalone';
import axios from 'axios';
import '../styles/AlertGraph.css';

const AlertGraph = ({ alertId, onNodeClick }) => {
  const networkRef = useRef(null);
  const [network, setNetwork] = useState(null);
  const [loading, setLoading] = useState(false);
  const [statistics, setStatistics] = useState(null);
  const [depth, setDepth] = useState(2);
  const [selectedNode, setSelectedNode] = useState(null);

  // 颜色方案
  const nodeColors = {
    Alert: {
      critical: '#ff0000',
      high: '#ff6600',
      medium: '#ffaa00',
      low: '#ffdd00',
      info: '#00aa00'
    },
    Device: '#3366cc',
    Process: '#66cc66',
    IOC: '#cc3366',
    User: '#cc9933'
  };

  // 网络配置
  const networkOptions = {
    nodes: {
      shape: 'dot',
      font: {
        size: 12,
        color: '#ffffff',
        strokeWidth: 2,
        strokeColor: '#000000'
      },
      borderWidth: 2,
      shadow: true
    },
    edges: {
      arrows: {
        to: {
          enabled: true,
          scaleFactor: 0.5
        }
      },
      smooth: {
        type: 'cubicBezier',
        roundness: 0.5
      },
      color: {
        inherit: false,
        color: '#848484',
        highlight: '#ff6600',
        hover: '#ff6600'
      },
      width: 2,
      shadow: true
    },
    physics: {
      enabled: true,
      solver: 'forceAtlas2Based',
      forceAtlas2Based: {
        gravitationalConstant: -50,
        centralGravity: 0.01,
        springLength: 100,
        springConstant: 0.08,
        damping: 0.4,
        avoidOverlap: 0.5
      },
      stabilization: {
        enabled: true,
        iterations: 1000,
        updateInterval: 100,
        fit: true
      }
    },
    interaction: {
      hover: true,
      tooltipDelay: 200,
      navigationButtons: true,
      keyboard: {
        enabled: true,
        speed: { x: 10, y: 10, zoom: 0.02 }
      }
    },
    layout: {
      improvedLayout: true,
      clusterThreshold: 150
    }
  };

  // 生成模拟图谱数据
  const generateMockGraphData = (alertId) => {
    const nodes = [];
    const edges = [];
    
    // 中心告警节点
    nodes.push({
      id: alertId,
      label: `Alert: ${alertId}`,
      title: `Alert ID: ${alertId}\nSeverity: High\nType: EDR`,
      node_type: 'Alert',
      group: 7,
      size: 30,
      properties: {
        severity: 7,
        type: 'EDR',
        message: 'Suspicious activity detected'
      }
    });
    
    // 添加设备节点
    const deviceId = `device-${alertId}`;
    nodes.push({
      id: deviceId,
      label: 'Device: WIN-SERVER-01',
      title: 'Device: WIN-SERVER-01\nIP: 192.168.1.100',
      node_type: 'Device',
      group: 'Device',
      size: 25,
      properties: {
        device_name: 'WIN-SERVER-01',
        ip: '192.168.1.100'
      }
    });
    
    edges.push({
      id: `edge-1`,
      from: alertId,
      to: deviceId,
      label: 'TRIGGERED_ON',
      edge_type: 'TRIGGERED_ON',
      width: 2,
      arrows: 'to'
    });
    
    // 添加进程节点
    const processId = `process-${alertId}`;
    nodes.push({
      id: processId,
      label: 'Process: powershell.exe',
      title: 'Process: powershell.exe\nPID: 1234',
      node_type: 'Process',
      group: 'Process',
      size: 20,
      properties: {
        process_name: 'powershell.exe',
        pid: 1234
      }
    });
    
    edges.push({
      id: `edge-2`,
      from: alertId,
      to: processId,
      label: 'INVOLVES',
      edge_type: 'INVOLVES_PROCESS',
      width: 2,
      arrows: 'to'
    });
    
    // 添加相关告警
    if (depth > 1) {
      for (let i = 1; i <= 3; i++) {
        const relatedAlertId = `${alertId}-related-${i}`;
        nodes.push({
          id: relatedAlertId,
          label: `Related Alert ${i}`,
          title: `Related Alert\nCorrelation: 0.${7 + i}`,
          node_type: 'Alert',
          group: 5,
          size: 25,
          properties: {
            severity: 5,
            correlation_score: 0.7 + i * 0.1
          }
        });
        
        edges.push({
          id: `edge-related-${i}`,
          from: alertId,
          to: relatedAlertId,
          label: 'CORRELATED',
          edge_type: 'CORRELATED_WITH',
          width: 1.5,
          arrows: 'to',
          dashes: true
        });
      }
    }
    
    // 添加IOC节点
    if (Math.random() > 0.5) {
      const iocId = `ioc-${alertId}`;
      nodes.push({
        id: iocId,
        label: 'IOC: Malicious IP',
        title: 'IOC: 185.220.101.45\nThreat Score: 95',
        node_type: 'IOC',
        group: 'IOC',
        size: 20,
        properties: {
          ioc_value: '185.220.101.45',
          threat_score: 95
        }
      });
      
      edges.push({
        id: `edge-ioc`,
        from: alertId,
        to: iocId,
        label: 'CONTAINS',
        edge_type: 'CONTAINS_IOC',
        width: 2,
        arrows: 'to'
      });
    }
    
    return {
      nodes,
      edges,
      statistics: {
        total_nodes: nodes.length,
        total_edges: edges.length,
        node_types: {
          Alert: nodes.filter(n => n.node_type === 'Alert').length,
          Device: nodes.filter(n => n.node_type === 'Device').length,
          Process: nodes.filter(n => n.node_type === 'Process').length,
          IOC: nodes.filter(n => n.node_type === 'IOC').length
        },
        edge_types: {},
        max_severity: 7
      }
    };
  };

  // 加载图谱数据
  const loadGraphData = async () => {
    if (!alertId) return;
    
    setLoading(true);
    try {
      // 先尝试从API获取
      const response = await axios.get(`/api/graph/alert/${alertId}`, {
        params: { depth },
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('sessionToken')}`
        }
      }).catch(err => {
        console.log('API not available, using mock data');
        return null;
      });

      if (response && response.data && response.data.success) {
        displayGraph(response.data.data);
        setStatistics(response.data.data.statistics);
      } else {
        // 使用模拟数据
        console.log('Using mock graph data');
        const mockData = generateMockGraphData(alertId);
        displayGraph(mockData);
        setStatistics(mockData.statistics);
      }
    } catch (error) {
      console.error('Error loading graph:', error);
      // 出错时使用模拟数据
      const mockData = generateMockGraphData(alertId);
      displayGraph(mockData);
      setStatistics(mockData.statistics);
    } finally {
      setLoading(false);
    }
  };

  // 显示图谱
  const displayGraph = (graphData) => {
    if (!networkRef.current) return;

    // 转换节点数据
    const nodes = new DataSet(
      graphData.nodes.map(node => ({
        id: node.id,
        label: node.label,
        title: node.title,
        group: node.node_type,
        size: node.size || 25,
        color: getNodeColor(node),
        font: {
          color: node.node_type === 'Alert' ? '#ffffff' : '#000000'
        },
        ...node.properties
      }))
    );

    // 转换边数据
    const edges = new DataSet(
      graphData.edges.map(edge => ({
        id: edge.id,
        from: edge.from,
        to: edge.to,
        label: edge.label,
        width: edge.width || 2,
        arrows: edge.arrows || 'to',
        dashes: edge.dashes || false,
        color: getEdgeColor(edge.edge_type)
      }))
    );

    // 创建网络
    const data = { nodes, edges };
    
    if (network) {
      network.setData(data);
    } else {
      const newNetwork = new Network(networkRef.current, data, networkOptions);
      
      // 事件处理
      newNetwork.on('click', (params) => {
        if (params.nodes.length > 0) {
          const nodeId = params.nodes[0];
          const node = nodes.get(nodeId);
          setSelectedNode(node);
          if (onNodeClick) {
            onNodeClick(node);
          }
        }
      });

      newNetwork.on('doubleClick', (params) => {
        if (params.nodes.length > 0) {
          const nodeId = params.nodes[0];
          expandNode(nodeId);
        }
      });

      newNetwork.on('stabilizationIterationsDone', () => {
        newNetwork.setOptions({ physics: false });
      });

      setNetwork(newNetwork);
    }
  };

  // 获取节点颜色
  const getNodeColor = (node) => {
    if (node.node_type === 'Alert') {
      const severity = node.group || 0;
      if (severity >= 8) return nodeColors.Alert.critical;
      if (severity >= 6) return nodeColors.Alert.high;
      if (severity >= 4) return nodeColors.Alert.medium;
      if (severity >= 2) return nodeColors.Alert.low;
      return nodeColors.Alert.info;
    }
    return nodeColors[node.node_type] || '#999999';
  };

  // 获取边颜色
  const getEdgeColor = (edgeType) => {
    const edgeColors = {
      'TRIGGERED_ON': '#3366cc',
      'INVOLVES_PROCESS': '#66cc66',
      'PARENT_OF': '#cc9933',
      'CORRELATED_WITH': '#ff6600',
      'CONTAINS_IOC': '#cc3366'
    };
    return edgeColors[edgeType] || '#848484';
  };

  // 展开节点
  const expandNode = async (nodeId) => {
    try {
      const response = await axios.get(`/api/graph/alert/${nodeId}`, {
        params: { depth: 1 },
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('sessionToken')}`
        }
      });

      if (response.data.success) {
        // 合并新数据到现有图谱
        mergeGraphData(response.data.data);
      }
    } catch (error) {
      console.error('Error expanding node:', error);
    }
  };

  // 合并图谱数据
  const mergeGraphData = (newData) => {
    if (!network) return;

    const { nodes, edges } = network.body.data;
    
    // 添加新节点
    newData.nodes.forEach(node => {
      if (!nodes.get(node.id)) {
        nodes.add({
          id: node.id,
          label: node.label,
          title: node.title,
          group: node.node_type,
          size: node.size || 25,
          color: getNodeColor(node),
          ...node.properties
        });
      }
    });

    // 添加新边
    newData.edges.forEach(edge => {
      if (!edges.get(edge.id)) {
        edges.add({
          id: edge.id,
          from: edge.from,
          to: edge.to,
          label: edge.label,
          width: edge.width || 2,
          arrows: edge.arrows || 'to',
          color: getEdgeColor(edge.edge_type)
        });
      }
    });
  };

  // 自动关联告警
  const correlateAlerts = async () => {
    setLoading(true);
    try {
      const response = await axios.post('/api/graph/correlate', 
        { time_window_hours: 24 },
        {
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('sessionToken')}`
          }
        }
      );

      if (response.data.success) {
        alert(`成功关联 ${response.data.data} 个告警`);
        loadGraphData(); // 重新加载图谱
      }
    } catch (error) {
      console.error('Error correlating alerts:', error);
    } finally {
      setLoading(false);
    }
  };

  // 检测横向移动
  const detectLateralMovement = async () => {
    try {
      const response = await axios.get('/api/graph/lateral-movement', {
        params: { 
          org_key: 'default',
          hours: 24 
        },
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('sessionToken')}`
        }
      });

      if (response.data.success && response.data.data.length > 0) {
        alert('检测到横向移动:\n' + response.data.data.join('\n'));
      } else {
        alert('未检测到横向移动');
      }
    } catch (error) {
      console.error('Error detecting lateral movement:', error);
    }
  };

  // 布局控制
  const changeLayout = (layoutType) => {
    if (!network) return;

    const layoutOptions = {
      hierarchical: {
        enabled: true,
        direction: 'UD',
        sortMethod: 'directed',
        nodeSpacing: 150,
        levelSeparation: 150
      }
    };

    if (layoutType === 'hierarchical') {
      network.setOptions({ layout: layoutOptions });
    } else {
      network.setOptions({ 
        layout: { 
          hierarchical: { enabled: false } 
        } 
      });
    }
  };

  useEffect(() => {
    loadGraphData();
  }, [alertId, depth]);

  useEffect(() => {
    return () => {
      if (network) {
        network.destroy();
      }
    };
  }, []);

  return (
    <div className="alert-graph-container">
      {loading && (
        <div className="loading-overlay">
          <div className="loading-spinner"></div>
          <p>正在加载图谱...</p>
        </div>
      )}
      
      <div className="graph-controls">
        <div className="control-group">
          <label>深度:</label>
          <select value={depth} onChange={(e) => setDepth(Number(e.target.value))}>
            <option value={1}>1层</option>
            <option value={2}>2层</option>
            <option value={3}>3层</option>
            <option value={4}>4层</option>
            <option value={5}>5层</option>
          </select>
        </div>
        
        <button onClick={() => loadGraphData()} disabled={loading}>
          {loading ? '加载中...' : '刷新'}
        </button>
        
        <button onClick={correlateAlerts} disabled={loading}>
          自动关联
        </button>
        
        <button onClick={detectLateralMovement}>
          检测横向移动
        </button>
        
        <button onClick={() => changeLayout('hierarchical')}>
          层级布局
        </button>
        
        <button onClick={() => changeLayout('physics')}>
          物理布局
        </button>
        
        <button onClick={() => network?.fit()}>
          适应窗口
        </button>
      </div>

      {statistics && (
        <div className="graph-statistics">
          <span>节点: {statistics.total_nodes}</span>
          <span>边: {statistics.total_edges}</span>
          <span>最高严重度: {statistics.max_severity}</span>
          {Object.entries(statistics.node_types).map(([type, count]) => (
            <span key={type}>{type}: {count}</span>
          ))}
        </div>
      )}

      <div ref={networkRef} className="graph-canvas" />

      {selectedNode && (
        <div className="node-details">
          <div className="node-details-header">
            <h3>节点详情</h3>
            <button 
              className="close-btn" 
              onClick={() => setSelectedNode(null)}
              title="关闭"
            >
              ✕
            </button>
          </div>
          <div className="detail-item">
            <strong>ID:</strong> {selectedNode.id}
          </div>
          <div className="detail-item">
            <strong>类型:</strong> {selectedNode.group}
          </div>
          <div className="detail-item">
            <strong>标签:</strong> {selectedNode.label}
          </div>
          {Object.entries(selectedNode).map(([key, value]) => {
            if (!['id', 'label', 'group', 'title', 'x', 'y', 'color', 'size', 'font'].includes(key)) {
              return (
                <div key={key} className="detail-item">
                  <strong>{key}:</strong> {JSON.stringify(value)}
                </div>
              );
            }
            return null;
          })}
        </div>
      )}
    </div>
  );
};

export default AlertGraph;