import React, { useState, useEffect } from 'react';
import { Card } from '../components/Card';
import { Button } from '../components/Button';
import { Alert } from '../components/Alert';
import { Loading } from '../components/Loading';
import { Badge } from '../components/Badge';
import { apiClient } from '../api/client';
import type { AuditLogEntry } from '../types';
import { formatDate, formatDuration } from '../utils/format';

export const AuditLogs: React.FC = () => {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [logs, setLogs] = useState<AuditLogEntry[]>([]);
  const [total, setTotal] = useState(0);
  const [filters, setFilters] = useState({
    user_id: '',
    start_date: '',
    end_date: '',
  });
  const [pagination, setPagination] = useState({
    limit: 20,
    offset: 0,
  });
  const [expandedLog, setExpandedLog] = useState<string | null>(null);

  useEffect(() => {
    loadAuditLogs();
  }, [pagination]);

  const loadAuditLogs = async () => {
    setLoading(true);
    setError(null);

    try {
      const params = {
        ...pagination,
        ...(filters.user_id && { user_id: filters.user_id }),
        ...(filters.start_date && { start_date: filters.start_date }),
        ...(filters.end_date && { end_date: filters.end_date }),
      };

      const data = await apiClient.getAuditLogs(params);
      setLogs(data.entries);
      setTotal(data.total);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load audit logs');
    } finally {
      setLoading(false);
    }
  };

  const handleFilterSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setPagination({ ...pagination, offset: 0 });
    loadAuditLogs();
  };

  const handleResetFilters = () => {
    setFilters({
      user_id: '',
      start_date: '',
      end_date: '',
    });
    setPagination({ limit: 20, offset: 0 });
  };

  const handleNextPage = () => {
    setPagination({
      ...pagination,
      offset: pagination.offset + pagination.limit,
    });
  };

  const handlePreviousPage = () => {
    setPagination({
      ...pagination,
      offset: Math.max(0, pagination.offset - pagination.limit),
    });
  };

  const toggleLogExpansion = (logId: string) => {
    setExpandedLog(expandedLog === logId ? null : logId);
  };

  const currentPage = Math.floor(pagination.offset / pagination.limit) + 1;
  const totalPages = Math.ceil(total / pagination.limit);

  return (
    <div className="max-w-7xl mx-auto">
      <div className="mb-8">
        <h2 className="text-3xl font-bold text-gray-900 mb-2">Audit Logs</h2>
        <p className="text-gray-600">
          View and search through the complete audit trail of all processed queries
        </p>
      </div>

      {error && (
        <div className="mb-6">
          <Alert type="error" message={error} onClose={() => setError(null)} />
        </div>
      )}

      {/* Filters */}
      <Card title="Filters" className="mb-6">
        <form onSubmit={handleFilterSubmit}>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
            <div>
              <label htmlFor="userId" className="block text-sm font-medium text-gray-700 mb-2">
                User ID
              </label>
              <input
                type="text"
                id="userId"
                value={filters.user_id}
                onChange={(e) => setFilters({ ...filters, user_id: e.target.value })}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="Filter by user ID"
              />
            </div>
            <div>
              <label htmlFor="startDate" className="block text-sm font-medium text-gray-700 mb-2">
                Start Date
              </label>
              <input
                type="datetime-local"
                id="startDate"
                value={filters.start_date}
                onChange={(e) => setFilters({ ...filters, start_date: e.target.value })}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
            <div>
              <label htmlFor="endDate" className="block text-sm font-medium text-gray-700 mb-2">
                End Date
              </label>
              <input
                type="datetime-local"
                id="endDate"
                value={filters.end_date}
                onChange={(e) => setFilters({ ...filters, end_date: e.target.value })}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
          </div>
          <div className="flex gap-3">
            <Button type="submit" variant="primary">
              Apply Filters
            </Button>
            <Button type="button" variant="outline" onClick={handleResetFilters}>
              Reset
            </Button>
          </div>
        </form>
      </Card>

      {/* Logs Table */}
      {loading ? (
        <Card>
          <Loading text="Loading audit logs..." />
        </Card>
      ) : logs.length === 0 ? (
        <Card>
          <Alert type="info" message="No audit logs found matching your criteria" />
        </Card>
      ) : (
        <>
          <Card>
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Timestamp
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      User ID
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Intent Type
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Decision
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Execution Time
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {logs.map((log) => (
                    <React.Fragment key={log.id}>
                      <tr className="hover:bg-gray-50">
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatDate(log.timestamp)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {log.user_id}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {log.intent_type}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <Badge status={log.decision} />
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatDuration(log.execution_time_ms)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm">
                          <button
                            onClick={() => toggleLogExpansion(log.id)}
                            className="text-blue-600 hover:text-blue-800 font-medium"
                          >
                            {expandedLog === log.id ? 'Hide' : 'Details'}
                          </button>
                        </td>
                      </tr>
                      {expandedLog === log.id && (
                        <tr>
                          <td colSpan={6} className="px-6 py-4 bg-gray-50">
                            <div className="space-y-3">
                              <div>
                                <span className="text-sm font-medium text-gray-700">Request ID:</span>
                                <span className="ml-2 text-sm font-mono text-gray-900">
                                  {log.request_id}
                                </span>
                              </div>
                              <div>
                                <span className="text-sm font-medium text-gray-700">Query:</span>
                                <p className="mt-1 text-sm text-gray-900">&quot;{log.query}&quot;</p>
                              </div>
                              {log.supervisor_id && (
                                <div>
                                  <span className="text-sm font-medium text-gray-700">
                                    Supervisor ID:
                                  </span>
                                  <span className="ml-2 text-sm text-gray-900">{log.supervisor_id}</span>
                                </div>
                              )}
                              <div>
                                <span className="text-sm font-medium text-gray-700 block mb-2">
                                  Metadata:
                                </span>
                                <div className="bg-white rounded p-3 border border-gray-200">
                                  <pre className="text-xs text-gray-800 overflow-x-auto">
                                    {JSON.stringify(log.metadata, null, 2)}
                                  </pre>
                                </div>
                              </div>
                            </div>
                          </td>
                        </tr>
                      )}
                    </React.Fragment>
                  ))}
                </tbody>
              </table>
            </div>
          </Card>

          {/* Pagination */}
          <div className="mt-6 flex items-center justify-between">
            <div className="text-sm text-gray-700">
              Showing {pagination.offset + 1} to {Math.min(pagination.offset + pagination.limit, total)} of{' '}
              {total} results
            </div>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={handlePreviousPage}
                disabled={pagination.offset === 0}
              >
                Previous
              </Button>
              <span className="px-4 py-2 text-sm text-gray-700">
                Page {currentPage} of {totalPages}
              </span>
              <Button
                variant="outline"
                size="sm"
                onClick={handleNextPage}
                disabled={pagination.offset + pagination.limit >= total}
              >
                Next
              </Button>
            </div>
          </div>
        </>
      )}
    </div>
  );
};
