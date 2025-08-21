import React, { useEffect, useRef, useState } from 'react';
import { Network } from 'vis-network/standalone';
import { DataSet } from 'vis-data/standalone';

const SimpleAlertGraph = ({ alertId, alertData }) => {
  const containerRef = useRef(null);
  const [network, setNetwork] = useState(null);

  useEffect(() => {
    if (!containerRef.current) return;

    // 根据威胁等级设置颜色
    const getSeverityColor = (severity) => {
      if (severity >= 8) return '#dc3545'; // 严重 - 红色
      if (severity >= 6) return '#fd7e14'; // 高 - 橙色
      if (severity >= 4) return '#ffc107'; // 中 - 黄色
      if (severity >= 2) return '#28a745'; // 低 - 绿色
      return '#6c757d'; // 信息 - 灰色
    };

    // 根据真实数据创建节点标签
    const alertLabel = alertData ? 
      `${alertData.type || alertData.alert_type}: ${(alertData.message || '').substring(0, 30)}...` : 
      `告警: ${alertId || 'ALERT-001'}`;
    
    const deviceLabel = alertData?.device_name || alertData?.device_id || 'Unknown Device';
    const processLabel = alertData?.process_name || alertData?.process_path || 'Unknown Process';
    const iocLabel = alertData?.ioc_hit || alertData?.ioc_id || '恶意IOC';
    
    const alertColor = alertData ? getSeverityColor(alertData.severity || 5) : '#ff6600';
    
    const nodes = new DataSet([
      { 
        id: 1, 
        label: alertLabel, 
        color: {
          background: alertColor,
          border: alertColor,
          highlight: {
            background: alertColor,
            border: '#000000'
          }
        },
        font: { 
          color: alertData && alertData.severity >= 6 ? '#ffffff' : '#000000', 
          size: 14, 
          face: 'arial' 
        },
        size: 35,
        title: alertData ? `威胁等级: ${alertData.severity}\n时间: ${alertData.create_time}` : ''
      },
      { 
        id: 2, 
        label: `设备: ${deviceLabel}`, 
        color: {
          background: '#3366cc',
          border: '#2255aa'
        },
        font: { color: '#ffffff', size: 12 },
        size: 25,
        title: alertData?.device_id || ''
      },
      { 
        id: 3, 
        label: `进程: ${processLabel}`, 
        color: {
          background: '#66cc66',
          border: '#55aa55'
        },
        font: { color: '#000000', size: 12 },
        size: 20,
        title: alertData?.process_path || ''
      },
      { 
        id: 4, 
        label: `IOC: ${iocLabel}`, 
        color: {
          background: '#cc3366',
          border: '#aa2255'
        },
        font: { color: '#ffffff', size: 12 },
        size: 20,
        title: alertData?.ioc_id || ''
      },
      { 
        id: 5, 
        label: '相关告警-1', 
        color: {
          background: '#ffaa00',
          border: '#dd9900'
        },
        font: { color: '#000000', size: 12 },
        size: 25 
      },
      { 
        id: 6, 
        label: '相关告警-2', 
        color: {
          background: '#ffaa00',
          border: '#dd9900'
        },
        font: { color: '#000000', size: 12 },
        size: 25 
      }
    ]);

    const edges = new DataSet([
      { 
        from: 1, 
        to: 2, 
        label: '触发于', 
        arrows: 'to',
        color: { color: '#848484' },
        font: { color: '#343434', size: 10 }
      },
      { 
        from: 1, 
        to: 3, 
        label: '涉及进程', 
        arrows: 'to',
        color: { color: '#848484' },
        font: { color: '#343434', size: 10 }
      },
      { 
        from: 1, 
        to: 4, 
        label: '包含IOC', 
        arrows: 'to',
        color: { color: '#848484' },
        font: { color: '#343434', size: 10 }
      },
      { 
        from: 1, 
        to: 5, 
        label: '关联', 
        arrows: 'to', 
        dashes: true,
        color: { color: '#999999' },
        font: { color: '#343434', size: 10 }
      },
      { 
        from: 1, 
        to: 6, 
        label: '关联', 
        arrows: 'to', 
        dashes: true,
        color: { color: '#999999' },
        font: { color: '#343434', size: 10 }
      }
    ]);

    const data = { nodes, edges };

    const options = {
      nodes: {
        shape: 'dot',
        borderWidth: 2,
        shadow: true,
        chosen: {
          node: function(values, id, selected, hovering) {
            values.shadowSize = 16;
          }
        }
      },
      edges: {
        width: 2,
        smooth: {
          type: 'continuous'
        },
        chosen: {
          edge: function(values, id, selected, hovering) {
            values.width = 3;
          }
        }
      },
      physics: {
        stabilization: false,
        barnesHut: {
          gravitationalConstant: -30000,
          springConstant: 0.04,
          springLength: 95
        }
      },
      interaction: {
        hover: true,
        tooltipDelay: 200
      }
    };

    // 创建网络图
    const newNetwork = new Network(containerRef.current, data, options);
    
    // 添加事件监听
    newNetwork.on('click', (params) => {
      if (params.nodes.length > 0) {
        console.log('Clicked node:', params.nodes[0]);
      }
    });

    setNetwork(newNetwork);

    // 清理函数
    return () => {
      if (newNetwork) {
        newNetwork.destroy();
      }
    };
  }, [alertId, alertData]);

  return (
    <div style={{ 
      width: '100%', 
      height: '500px', 
      border: '1px solid #ddd',
      borderRadius: '8px',
      backgroundColor: '#f8f9fa',
      position: 'relative'
    }}>
      <div 
        ref={containerRef} 
        style={{ 
          width: '100%', 
          height: '100%',
          backgroundColor: 'white'
        }}
      />
      <div style={{
        position: 'absolute',
        top: '10px',
        left: '10px',
        padding: '8px 12px',
        backgroundColor: 'rgba(255, 255, 255, 0.95)',
        border: '1px solid #e1e4e8',
        borderRadius: '6px',
        fontSize: '12px',
        color: '#586069',
        pointerEvents: 'none'
      }}>
        提示：拖拽节点移动，滚轮缩放，点击节点查看详情
      </div>
    </div>
  );
};

export default SimpleAlertGraph;