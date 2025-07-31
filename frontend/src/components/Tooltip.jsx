import React from 'react';

const Tooltip = ({ tooltip }) => {
  if (!tooltip.show) return null;

  return (
    <div 
      className="tooltip"
      style={{
        left: tooltip.x,
        top: tooltip.y,
        transform: 'translateX(-50%) translateY(-100%)'
      }}
    >
      <div className="tooltip-content">
        <h4>{tooltip.content.name}</h4>
        <p><strong>服务器：</strong> {tooltip.content.bootstrap_servers}</p>
        <p><strong>主题：</strong> {tooltip.content.topic}</p>
        <p><strong>消费者组：</strong> {tooltip.content.group_id}</p>
        <p><strong>超时时间：</strong> {tooltip.content.message_timeout_ms}ms</p>
        <p><strong>重试次数：</strong> {tooltip.content.retries}</p>
        <p><strong>偏移重置：</strong> {tooltip.content.auto_offset_reset}</p>
        <p><strong>自动提交：</strong> {tooltip.content.enable_auto_commit ? '是' : '否'}</p>
      </div>
    </div>
  );
};

export default Tooltip;