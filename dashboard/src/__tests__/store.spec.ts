import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useRunsStore } from '@/stores/runs'
import { fetchRuns, fetchRun, compareRuns } from '@/api/client'
import type { RunSummary, RunDetail, CompareResponse } from '@/types'

// 替换真实的 API 调用为 mock，store 测试只关心状态流转
vi.mock('@/api/client', () => ({
  fetchRuns: vi.fn(),
  fetchRun: vi.fn(),
  compareRuns: vi.fn(),
}))

const mockedFetchRuns = vi.mocked(fetchRuns)
const mockedFetchRun = vi.mocked(fetchRun)
const mockedCompareRuns = vi.mocked(compareRuns)

const runSummary: RunSummary = {
  run_id: 'run-1',
  task_name: 'demo',
  created_at: '2026-07-31T10:00:00Z',
  model: 'gpt-4o',
  status: 'success',
  safety_score: 95,
  total_turns: 5,
  total_duration_ms: 1500,
}

const runDetail: RunDetail = {
  run_id: 'run-1',
  task_name: 'demo',
  created_at: '2026-07-31T10:00:00Z',
  model: 'gpt-4o',
  system_prompt: '',
  max_turns: 10,
  status: 'success',
  total_turns: 5,
  total_tokens: 120,
  total_duration_ms: 1500,
  safety_score: 95,
  tool_calls: [],
  alerts: [],
  events_json: '[]',
}

describe('runs store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('loadRuns populates runs and total', async () => {
    mockedFetchRuns.mockResolvedValue({ runs: [runSummary], total: 1 })
    const store = useRunsStore()
    await store.loadRuns()
    expect(store.runs).toEqual([runSummary])
    expect(store.total).toBe(1)
  })

  it('loadRuns forwards query params to the API', async () => {
    mockedFetchRuns.mockResolvedValue({ runs: [], total: 0 })
    const store = useRunsStore()
    await store.loadRuns({ task_name: 'demo', min_score: 80, limit: 20, offset: 0 })
    expect(mockedFetchRuns).toHaveBeenCalledWith({
      task_name: 'demo',
      min_score: 80,
      limit: 20,
      offset: 0,
    })
  })

  it('loadRun populates currentRun', async () => {
    mockedFetchRun.mockResolvedValue(runDetail)
    const store = useRunsStore()
    await store.loadRun('run-1')
    expect(store.currentRun).toEqual(runDetail)
  })

  it('loadCompare populates comparedRuns and comparison', async () => {
    const res: CompareResponse = {
      runs: [runDetail],
      comparison: { score_diff: 10, turns_diff: 1, tokens_diff: 100, duration_diff_ms: 500 },
    }
    mockedCompareRuns.mockResolvedValue(res)
    const store = useRunsStore()
    await store.loadCompare(['run-1', 'run-2'])
    expect(store.comparedRuns).toEqual([runDetail])
    expect(store.comparison).toEqual(res.comparison)
  })

  it('sets loading true while a request is in flight', async () => {
    let resolveFetch: (value: { runs: RunSummary[]; total: number }) => void
    mockedFetchRuns.mockReturnValue(
      new Promise((resolve) => {
        resolveFetch = resolve
      }),
    )
    const store = useRunsStore()
    const pending = store.loadRuns()
    expect(store.loading).toBe(true)
    resolveFetch!({ runs: [runSummary], total: 1 })
    await pending
    expect(store.loading).toBe(false)
  })

  it('resets loading even when the request fails', async () => {
    mockedFetchRuns.mockRejectedValue(new Error('boom'))
    const store = useRunsStore()
    await expect(store.loadRuns()).rejects.toThrow('boom')
    expect(store.loading).toBe(false)
  })
})
