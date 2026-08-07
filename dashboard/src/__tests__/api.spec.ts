import { describe, it, expect, vi, beforeEach } from 'vitest'

// axios 默认导出对象，mock create() 返回我们可控的实例
const { apiMock } = vi.hoisted(() => ({
  apiMock: { get: vi.fn(), post: vi.fn() },
}))

vi.mock('axios', () => ({
  default: { create: vi.fn(() => apiMock) },
}))

import { fetchRuns, fetchRun, uploadRun, compareRuns } from '@/api/client'
import type { UploadRequest } from '@/types'

describe('api client', () => {
  beforeEach(() => vi.clearAllMocks())

  it('fetchRuns sends query params', async () => {
    apiMock.get.mockResolvedValue({ data: { runs: [], total: 0 } })
    await fetchRuns({ task_name: 'demo', min_score: 80, limit: 10, offset: 0 })
    expect(apiMock.get).toHaveBeenCalledWith('/runs', {
      params: { task_name: 'demo', min_score: 80, limit: 10, offset: 0 },
    })
  })

  it('fetchRuns omits params when called without args', async () => {
    apiMock.get.mockResolvedValue({ data: { runs: [], total: 0 } })
    await fetchRuns()
    expect(apiMock.get).toHaveBeenCalledWith('/runs', { params: undefined })
  })

  it('fetchRun requests a single run by id', async () => {
    apiMock.get.mockResolvedValue({ data: {} })
    await fetchRun('run-1')
    expect(apiMock.get).toHaveBeenCalledWith('/runs/run-1')
  })

  it('compareRuns joins ids into a single query param', async () => {
    apiMock.get.mockResolvedValue({ data: { runs: [], comparison: {} } })
    await compareRuns(['a', 'b', 'c'])
    expect(apiMock.get).toHaveBeenCalledWith('/runs/compare', {
      params: { ids: 'a,b,c' },
    })
  })

  it('uploadRun posts the request body', async () => {
    const req: UploadRequest = {
      task_name: 'demo',
      model: 'gpt-4o',
      system_prompt: '',
      max_turns: 10,
      events_json: '[]',
      status: 'success',
      total_turns: 1,
      total_tokens: 100,
      total_duration_ms: 1000,
    }
    apiMock.post.mockResolvedValue({ data: {} })
    await uploadRun(req)
    expect(apiMock.post).toHaveBeenCalledWith('/runs', req)
  })
})
