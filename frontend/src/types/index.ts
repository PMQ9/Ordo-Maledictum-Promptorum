// API Types for Intent Segregation System

export interface IntentRequest {
  user_id: string;
  query: string;
}

export interface ParsedIntent {
  intent_type: string;
  entities: Record<string, any>;
  confidence: number;
  raw_query: string;
}

export interface ModelVote {
  model_name: string;
  vote: 'allow' | 'deny' | 'uncertain';
  confidence: number;
  reasoning: string;
}

export interface VotingResult {
  final_decision: 'allow' | 'deny' | 'requires_approval';
  votes: ModelVote[];
  consensus_score: number;
  timestamp: string;
}

export interface ComparisonResult {
  matches: boolean;
  similarity_score: number;
  differences: string[];
  reasoning: string;
}

export interface ProcessingResult {
  request_id: string;
  user_id: string;
  query: string;
  parsed_intent: ParsedIntent;
  voting_result: VotingResult;
  comparison_result: ComparisonResult;
  final_status: 'allowed' | 'denied' | 'pending_approval';
  timestamp: string;
  execution_result?: any;
}

export interface ApprovalRequest {
  request_id: string;
  user_id: string;
  query: string;
  parsed_intent: ParsedIntent;
  voting_result: VotingResult;
  comparison_result: ComparisonResult;
  submitted_at: string;
  status: 'pending' | 'approved' | 'rejected';
}

export interface ApprovalDecision {
  request_id: string;
  supervisor_id: string;
  decision: 'approve' | 'reject';
  notes?: string;
}

export interface AuditLogEntry {
  id: string;
  request_id: string;
  user_id: string;
  query: string;
  intent_type: string;
  decision: string;
  supervisor_id?: string;
  timestamp: string;
  execution_time_ms: number;
  metadata: Record<string, any>;
}

export interface ApiError {
  error: string;
  details?: string;
  timestamp: string;
}

export interface HealthStatus {
  status: 'healthy' | 'unhealthy';
  version: string;
  uptime: number;
}
