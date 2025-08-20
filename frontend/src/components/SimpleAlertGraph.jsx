import React, { useEffect, useRef, useState } from 'react';
import { Network } from 'vis-network/standalone';
import { DataSet } from 'vis-data/standalone';

const SimpleAlertGraph = ({ alertId }) => {
  const containerRef = useRef(null);
  const [network, setNetwork] = useState(null);

  useEffect(() => {
    if (!containerRef.current) return;

    // 创建简单的测试数据 - 修复文字颜色
    const nodes = new DataSet([
      { 
        id: 1, 
        label: `告警: ${alertId || 'ALERT-001'}`, 
        color: {
          background: '#ff6600',
          border: '#cc5200',
          highlight: {
            background: '#ff8833',
            border: '#ff6600'
          }
        },
        font: { color: '#000000', size: 14, face: 'arial' },
        size: 30 
      },
      { 
        id: 2, 
        label: '设备: WIN-SERVER', 
        color: {
          background: '#3366cc',
          border: '#2255aa'
        },
        font: { color: '#ffffff', size: 12 },
        size: 25 
      },
      { 
        id: 3, 
        label: '进程: powershell.exe', 
        color: {
          background: '#66cc66',
          border: '#55aa55'
        },
        font: { color: '#000000', size: 12 },
        size: 20 
      },
      { 
        id: 4, 
        label: 'IOC: 恶意IP', 
        color: {
          background: '#cc3366',
          border: '#aa2255'
        },
        font: { color: '#ffffff', size: 12 },
        size: 20 
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
  }, [alertId]);

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