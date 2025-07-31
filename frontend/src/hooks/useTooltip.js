import { useState } from 'react';

export const useTooltip = () => {
  const [tooltip, setTooltip] = useState({ show: false, content: '', x: 0, y: 0 });

  const showTooltip = (event, config) => {
    const rect = event.target.getBoundingClientRect();
    setTooltip({
      show: true,
      content: config,
      x: rect.left + rect.width / 2,
      y: rect.top - 10
    });
  };

  const hideTooltip = () => {
    setTooltip({ show: false, content: '', x: 0, y: 0 });
  };

  return {
    tooltip,
    showTooltip,
    hideTooltip
  };
};