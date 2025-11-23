import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Card } from '../components/Card';
import { Button } from '../components/Button';
import { Alert } from '../components/Alert';
import { Loading } from '../components/Loading';
import { IntentVisualization } from '../components/IntentVisualization';
import { Badge } from '../components/Badge';
import { apiClient } from '../api/client';
import type { ProcessingResult } from '../types';
import { formatDate } from '../utils/format';

export const QueryInterface: React.FC = () => {
  const navigate = useNavigate();
  const [userId, setUserId] = useState('');
  const [query, setQuery] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<ProcessingResult | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setResult(null);

    if (!userId.trim() || !query.trim()) {
      setError('Please provide both User ID and Query');
      return;
    }

    setLoading(true);

    try {
      const processingResult = await apiClient.submitQuery({
        user_id: userId,
        query: query,
      });
      setResult(processingResult);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to process query');
    } finally {
      setLoading(false);
    }
  };

  const handleReset = () => {
    setQuery('');
    setResult(null);
    setError(null);
  };

  const handleViewApproval = () => {
    if (result?.request_id) {
      navigate(`/approval/${result.request_id}`);
    }
  };

  return (
    <div className="max-w-6xl mx-auto">
      <div className="mb-8">
        <h2 className="text-3xl font-bold text-gray-900 mb-2">Query Processing Interface</h2>
        <p className="text-gray-600">
          Submit a query to be processed through the Intent Segregation security pipeline
        </p>
      </div>

      {error && (
        <div className="mb-6">
          <Alert type="error" message={error} onClose={() => setError(null)} />
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Input Form */}
        <div>
          <Card title="Submit Query">
            <form onSubmit={handleSubmit} className="space-y-4">
              <div>
                <label htmlFor="userId" className="block text-sm font-medium text-gray-700 mb-2">
                  User ID
                </label>
                <input
                  type="text"
                  id="userId"
                  value={userId}
                  onChange={(e) => setUserId(e.target.value)}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="Enter your user ID"
                  disabled={loading}
                />
              </div>
              <div>
                <label htmlFor="query" className="block text-sm font-medium text-gray-700 mb-2">
                  Query
                </label>
                <textarea
                  id="query"
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  rows={6}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="Enter your query here..."
                  disabled={loading}
                />
              </div>
              <div className="flex gap-3">
                <Button type="submit" variant="primary" className="flex-1" disabled={loading}>
                  {loading ? 'Processing...' : 'Submit Query'}
                </Button>
                {result && (
                  <Button type="button" variant="outline" onClick={handleReset}>
                    Reset
                  </Button>
                )}
              </div>
            </form>
          </Card>

          {/* Processing Status */}
          {loading && (
            <div className="mt-6">
              <Card>
                <Loading text="Processing your query through the security pipeline..." />
              </Card>
            </div>
          )}

          {/* Result Summary */}
          {result && !loading && (
            <div className="mt-6">
              <Card title="Processing Result">
                <div className="space-y-4">
                  <div className="flex items-center justify-between pb-4 border-b border-gray-200">
                    <span className="text-sm font-medium text-gray-700">Status:</span>
                    <Badge status={result.final_status} />
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium text-gray-700">Request ID:</span>
                    <span className="text-sm font-mono text-gray-900">{result.request_id}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium text-gray-700">Processed At:</span>
                    <span className="text-sm text-gray-900">{formatDate(result.timestamp)}</span>
                  </div>

                  {result.final_status === 'pending_approval' && (
                    <div className="pt-4 border-t border-gray-200">
                      <Alert
                        type="warning"
                        message="This query requires supervisor approval before execution."
                      />
                      <Button
                        variant="primary"
                        className="w-full mt-3"
                        onClick={handleViewApproval}
                      >
                        View Approval Request
                      </Button>
                    </div>
                  )}

                  {result.final_status === 'allowed' && result.execution_result && (
                    <div className="pt-4 border-t border-gray-200">
                      <span className="text-sm font-medium text-gray-700 block mb-2">
                        Execution Result:
                      </span>
                      <div className="bg-green-50 rounded p-3 border border-green-200">
                        <pre className="text-xs text-gray-800 overflow-x-auto">
                          {JSON.stringify(result.execution_result, null, 2)}
                        </pre>
                      </div>
                    </div>
                  )}

                  {result.final_status === 'denied' && (
                    <div className="pt-4 border-t border-gray-200">
                      <Alert
                        type="error"
                        message="This query has been denied by the security system."
                      />
                    </div>
                  )}
                </div>
              </Card>
            </div>
          )}
        </div>

        {/* Visualization */}
        <div>
          {result && !loading && (
            <IntentVisualization
              parsedIntent={result.parsed_intent}
              votingResult={result.voting_result}
              comparisonResult={result.comparison_result}
            />
          )}
        </div>
      </div>
    </div>
  );
};
