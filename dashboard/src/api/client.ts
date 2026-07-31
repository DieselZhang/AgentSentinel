import axios from 'axios'
import type { CompareResponse, RunDetail, RunListResponse, UploadRequest } from '@/types'

const api = axios.create({ baseURL: '/api' })

export async function fetchRuns(params?: {
  task_name?: string
  min_score?: number
  limit?: number
  offset?: number
}): Promise<RunListResponse> {
  const { data } = await api.get('/runs', { params })
  return data
}

export async function fetchRun(id: string): Promise<RunDetail> {
  const { data } = await api.get(`/runs/${id}`)
  return data
}

export async function uploadRun(req: UploadRequest): Promise<RunDetail> {
  const { data } = await api.post('/runs', req)
  return data
}

export async function compareRuns(ids: string[]): Promise<CompareResponse> {
  const { data } = await api.get('/runs/compare', { params: { ids: ids.join(',') } })
  return data
}
