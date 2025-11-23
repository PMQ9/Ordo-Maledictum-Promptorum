import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Card } from '../components/Card';
import { Button } from '../components/Button';
import { Alert } from '../components/Alert';
import { Loading } from '../components/Loading';
import { IntentVisualization } from '../components/IntentVisualization';
import { Badge } from '../components/Badge';
import { apiClient } from '../api/client';
import type { ApprovalRequest } from '../types';
import { formatDate } from '../utils/format';

export const ApprovalReview: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [approvalRequest, setApprovalRequest] = useState<ApprovalRequest | null>(null);
  const [supervisorId, setSupervisorId] = useState('');
  const [notes, setNotes] = useState('');

  useEffect(() => {
    if (id) {
      loadApprovalRequest(id);
    }
  }, [id]);

  const loadApprovalRequest = async (requestId: string) => {
    setLoading(true);
    setError(null);

    try {
      const data = await apiClient.getApprovalRequest(requestId);
      setApprovalRequest(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load approval request');
    } finally {
      setLoading(false);
    }
  };

  const handleDecision = async (decision: 'approve' | 'reject') => {
    if (!supervisorId.trim()) {
      setError('Please enter your Supervisor ID');
      return;
    }

    if (!id) return;

    setSubmitting(true);
    setError(null);
    setSuccess(null);

    try {
      const result = await apiClient.submitApprovalDecision({
        request_id: id,
        supervisor_id: supervisorId,
        decision,
        notes: notes.trim() || undefined,
      });

      setSuccess(result.message || `Request ${decision}d successfully`);

      // Redirect to home after 2 seconds
      setTimeout(() => {
        navigate('/');
      }, 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : `Failed to ${decision} request`);
    } finally {
      setSubmitting(false);
    }
  };

  if (loading) {
    return (
      <div className="max-w-6xl mx-auto">
        <Card>
          <Loading text="Loading approval request..." />
        </Card>
      </div>
    );
  }

  if (!approvalRequest) {
    return (
      <div className="max-w-6xl mx-auto">
        <Alert type="error" message="Approval request not found" />
        <Button variant="outline" className="mt-4" onClick={() => navigate('/')}>
          Back to Home
        </Button>
      </div>
    );
  }

  const isCompleted = approvalRequest.status !== 'pending';

  return (
    <div className="max-w-6xl mx-auto">
      <div className="mb-8">
        <h2 className="text-3xl font-bold text-gray-900 mb-2">Approval Review</h2>
        <p className="text-gray-600">
          Review and approve or reject the following query request
        </p>
      </div>

      {error && (
        <div className="mb-6">
          <Alert type="error" message={error} onClose={() => setError(null)} />
        </div>
      )}

      {success && (
        <div className="mb-6">
          <Alert type="success" message={success} />
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Request Details */}
        <div className="space-y-6">
          <Card title="Request Information">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium text-gray-700">Status:</span>
                <Badge status={approvalRequest.status} />
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium text-gray-700">Request ID:</span>
                <span className="text-sm font-mono text-gray-900">{approvalRequest.request_id}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium text-gray-700">User ID:</span>
                <span className="text-sm text-gray-900">{approvalRequest.user_id}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium text-gray-700">Submitted At:</span>
                <span className="text-sm text-gray-900">{formatDate(approvalRequest.submitted_at)}</span>
              </div>
              <div className="pt-4 border-t border-gray-200">
                <span className="text-sm font-medium text-gray-700 block mb-2">Query:</span>
                <div className="bg-gray-50 rounded p-3 border border-gray-200">
                  <p className="text-sm text-gray-900">&quot;{approvalRequest.query}&quot;</p>
                </div>
              </div>
            </div>
          </Card>

          {!isCompleted && (
            <Card title="Supervisor Decision">
              <div className="space-y-4">
                <div>
                  <label htmlFor="supervisorId" className="block text-sm font-medium text-gray-700 mb-2">
                    Supervisor ID <span className="text-red-500">*</span>
                  </label>
                  <input
                    type="text"
                    id="supervisorId"
                    value={supervisorId}
                    onChange={(e) => setSupervisorId(e.target.value)}
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                    placeholder="Enter your supervisor ID"
                    disabled={submitting}
                  />
                </div>
                <div>
                  <label htmlFor="notes" className="block text-sm font-medium text-gray-700 mb-2">
                    Notes (Optional)
                  </label>
                  <textarea
                    id="notes"
                    value={notes}
                    onChange={(e) => setNotes(e.target.value)}
                    rows={4}
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                    placeholder="Add any notes or comments..."
                    disabled={submitting}
                  />
                </div>
                <div className="flex gap-3 pt-4">
                  <Button
                    variant="success"
                    className="flex-1"
                    onClick={() => handleDecision('approve')}
                    disabled={submitting}
                  >
                    {submitting ? 'Processing...' : 'Approve'}
                  </Button>
                  <Button
                    variant="danger"
                    className="flex-1"
                    onClick={() => handleDecision('reject')}
                    disabled={submitting}
                  >
                    {submitting ? 'Processing...' : 'Reject'}
                  </Button>
                </div>
              </div>
            </Card>
          )}

          {isCompleted && (
            <Card title="Decision Summary">
              <Alert
                type={approvalRequest.status === 'approved' ? 'success' : 'error'}
                message={`This request has been ${approvalRequest.status}`}
              />
              <Button variant="outline" className="w-full mt-4" onClick={() => navigate('/')}>
                Back to Home
              </Button>
            </Card>
          )}
        </div>

        {/* Visualization */}
        <div>
          <IntentVisualization
            parsedIntent={approvalRequest.parsed_intent}
            votingResult={approvalRequest.voting_result}
            comparisonResult={approvalRequest.comparison_result}
          />
        </div>
      </div>
    </div>
  );
};
