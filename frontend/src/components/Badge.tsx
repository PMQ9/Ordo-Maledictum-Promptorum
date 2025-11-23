import React from 'react';
import { getStatusColor } from '../utils/format';

interface BadgeProps {
  status: string;
  className?: string;
}

export const Badge: React.FC<BadgeProps> = ({ status, className = '' }) => {
  const colors = getStatusColor(status);

  return (
    <span
      className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium border ${colors.bg} ${colors.text} ${colors.border} ${className}`}
    >
      {status.replace(/_/g, ' ').toUpperCase()}
    </span>
  );
};
