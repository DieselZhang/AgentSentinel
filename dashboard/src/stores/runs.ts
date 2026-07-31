import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchRuns, fetchRun, compareRuns } from '@/api/client'
import type { CompareSummary, RunSummary, RunDetail } from '@/types'

export const useRunsStore = defineStore('runs', () => {
  const runs = ref<RunSummary[]>([])
  const total = ref(0)
  const loading = ref(false)
  const currentRun = ref<RunDetail | null>(null)
  const comparedRuns = ref<RunDetail[]>([])
  const comparison = ref<CompareSummary | null>(null)

  async function loadRuns(params?: {
    task_name?: string
    min_score?: number
    limit?: number
    offset?: number
  }) {
    loading.value = true
    try {
      const res = await fetchRuns(params)
      runs.value = res.runs
      total.value = res.total
    } finally {
      loading.value = false
    }
  }

  async function loadRun(id: string) {
    loading.value = true
    try {
      currentRun.value = await fetchRun(id)
    } finally {
      loading.value = false
    }
  }

  async function loadCompare(ids: string[]) {
    loading.value = true
    try {
      const res = await compareRuns(ids)
      comparedRuns.value = res.runs
      comparison.value = res.comparison
    } finally {
      loading.value = false
    }
  }

  return {
    runs,
    total,
    loading,
    currentRun,
    comparedRuns,
    comparison,
    loadRuns,
    loadRun,
    loadCompare,
  }
})
