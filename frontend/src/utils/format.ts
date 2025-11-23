// Utility functions for formatting data

export const formatDate = (dateString: string): string => {
  const date = new Date(dateString);
  return date.toLocaleString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
};

export const formatRelativeTime = (dateString: string): string => {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffSecs < 60) return `${diffSecs} seconds ago`;
  if (diffMins < 60) return `${diffMins} minutes ago`;
  if (diffHours < 24) return `${diffHours} hours ago`;
  if (diffDays < 7) return `${diffDays} days ago`;
  return formatDate(dateString);
};

export const formatPercentage = (value: number): string => {
  return `${(value * 100).toFixed(1)}%`;
};

export const formatDuration = (ms: number): string => {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`;
  return `${(ms / 60000).toFixed(2)}m`;
};

export const capitalizeFirst = (str: string): string => {
  return str.charAt(0).toUpperCase() + str.slice(1);
};

export const getStatusColor = (
  status: string
): { bg: string; text: string; border: string } => {
  const statusMap: Record<string, { bg: string; text: string; border: string }> = {
    allowed: { bg: 'bg-green-100', text: 'text-green-800', border: 'border-green-300' },
    approved: { bg: 'bg-green-100', text: 'text-green-800', border: 'border-green-300' },
    allow: { bg: 'bg-green-100', text: 'text-green-800', border: 'border-green-300' },
    denied: { bg: 'bg-red-100', text: 'text-red-800', border: 'border-red-300' },
    rejected: { bg: 'bg-red-100', text: 'text-red-800', border: 'border-red-300' },
    deny: { bg: 'bg-red-100', text: 'text-red-800', border: 'border-red-300' },
    pending: { bg: 'bg-yellow-100', text: 'text-yellow-800', border: 'border-yellow-300' },
    pending_approval: { bg: 'bg-yellow-100', text: 'text-yellow-800', border: 'border-yellow-300' },
    requires_approval: { bg: 'bg-yellow-100', text: 'text-yellow-800', border: 'border-yellow-300' },
    uncertain: { bg: 'bg-gray-100', text: 'text-gray-800', border: 'border-gray-300' },
  };

  return statusMap[status.toLowerCase()] || { bg: 'bg-gray-100', text: 'text-gray-800', border: 'border-gray-300' };
};
