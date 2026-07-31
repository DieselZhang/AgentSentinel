export interface RunSummary {
  run_id: string
  task_name: string
  created_at: string
  model: string
  status: string
  safety_score: number
  total_turns: number
  total_duration_ms: number
}

export interface ToolCallRecord {
  tool_name: string
  arguments: Record<string, unknown>
  result: string
  blocked: boolean
  is_error: boolean
  timestamp: string
}

export interface SafetyAlert {
  severity: 'low' | 'medium' | 'high' | 'critical'
  message: string
  event_index: number
}

export interface RunDetail {
  run_id: string
  task_name: string
  created_at: string
  model: string
  system_prompt: string
  max_turns: number
  status: string
  total_turns: number
  total_tokens: number
  total_duration_ms: number
  safety_score: number
  tool_calls: ToolCallRecord[]
  alerts: SafetyAlert[]
  events_json: string
}

export interface RunListResponse {
  runs: RunSummary[]
  total: number
}

export interface CompareSummary {
  score_diff: number
  turns_diff: number
  tokens_diff: number
  duration_diff_ms: number
}

export interface CompareResponse {
  runs: RunDetail[]
  comparison: CompareSummary
}

export interface UploadRequest {
  task_name: string
  model: string
  system_prompt: string
  max_turns: number
  events_json: string
  status: string
  total_turns: number
  total_tokens: number
  total_duration_ms: number
}
