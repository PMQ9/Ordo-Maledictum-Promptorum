import React from 'react';

interface AlertProps {
  type: 'success' | 'error' | 'warning' | 'info';
  message: string;
  onClose?: () => void;
}

export const Alert: React.FC<AlertProps> = ({ type, message, onClose }) => {
  const typeClasses = {
    success: 'bg-green-50 border-green-200 text-green-800',
    error: 'bg-red-50 border-red-200 text-red-800',
    warning: 'bg-yellow-50 border-yellow-200 text-yellow-800',
    info: 'bg-blue-50 border-blue-200 text-blue-800',
  };

  const iconMap = {
    success: '✓',
    error: '✕',
    warning: '⚠',
    info: 'ℹ',
  };

  return (
    <div className={`rounded-lg border-l-4 p-4 ${typeClasses[type]} flex items-start justify-between`}>
      <div className="flex items-start">
        <span className="text-xl mr-3">{iconMap[type]}</span>
        <p className="text-sm font-medium">{message}</p>
      </div>
      {onClose && (
        <button
          onClick={onClose}
          className="ml-4 text-gray-400 hover:text-gray-600 focus:outline-none"
        >
          <span className="text-xl">&times;</span>
        </button>
      )}
    </div>
  );
};
