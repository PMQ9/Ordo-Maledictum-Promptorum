import axios, { type AxiosInstance, type AxiosError } from 'axios';
import type {
  IntentRequest,
  ProcessingResult,
  ApprovalRequest,
  ApprovalDecision,
  AuditLogEntry,
  HealthStatus,
  ApiError,
} from '../types';

// Get API base URL from environment variable or use default
const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api';

class ApiClient {
  private client: AxiosInstance;

  constructor() {
    this.client = axios.create({
      baseURL: API_BASE_URL,
      headers: {
        'Content-Type': 'application/json',
      },
      timeout: 30000, // 30 seconds
    });

    // Add response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      (error: AxiosError<ApiError>) => {
        if (error.response) {
          // Server responded with error
          const apiError = error.response.data;
          throw new Error(apiError.error || 'An error occurred');
        } else if (error.request) {
          // Request made but no response
          throw new Error('No response from server. Please check your connection.');
        } else {
          // Error setting up request
          throw new Error(error.message || 'An unexpected error occurred');
        }
      }
    );
  }

  // Health check
  async checkHealth(): Promise<HealthStatus> {
    const response = await this.client.get<HealthStatus>('/health');
    return response.data;
  }

  // Submit a query for processing
  async submitQuery(request: IntentRequest): Promise<ProcessingResult> {
    const response = await this.client.post<ProcessingResult>('/process', request);
    return response.data;
  }

  // Get processing result by request ID
  async getProcessingResult(requestId: string): Promise<ProcessingResult> {
    const response = await this.client.get<ProcessingResult>(`/results/${requestId}`);
    return response.data;
  }

  // Get pending approval requests
  async getPendingApprovals(): Promise<ApprovalRequest[]> {
    const response = await this.client.get<ApprovalRequest[]>('/approvals/pending');
    return response.data;
  }

  // Get approval request by ID
  async getApprovalRequest(requestId: string): Promise<ApprovalRequest> {
    const response = await this.client.get<ApprovalRequest>(`/approvals/${requestId}`);
    return response.data;
  }

  // Submit approval decision
  async submitApprovalDecision(decision: ApprovalDecision): Promise<{ success: boolean; message: string }> {
    const response = await this.client.post(`/approvals/${decision.request_id}`, decision);
    return response.data;
  }

  // Get audit logs with optional filters
  async getAuditLogs(params?: {
    user_id?: string;
    start_date?: string;
    end_date?: string;
    limit?: number;
    offset?: number;
  }): Promise<{ entries: AuditLogEntry[]; total: number }> {
    const response = await this.client.get('/audit/logs', { params });
    return response.data;
  }

  // Get audit log entry by ID
  async getAuditLogEntry(id: string): Promise<AuditLogEntry> {
    const response = await this.client.get<AuditLogEntry>(`/audit/logs/${id}`);
    return response.data;
  }
}

// Export singleton instance
export const apiClient = new ApiClient();
export default apiClient;
