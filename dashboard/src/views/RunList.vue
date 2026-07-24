<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRunsStore } from '@/stores/runs'
import RunCard from '@/components/RunCard.vue'

const store = useRunsStore()

const search = ref('')
const minScore = ref(0)

let debounceTimer: ReturnType<typeof setTimeout> | null = null

function doLoad() {
  const params: { task_name?: string; min_score?: number; limit?: number; offset?: number } = {
    limit: 50,
    offset: 0,
  }
  if (search.value.trim()) params.task_name = search.value.trim()
  if (minScore.value > 0) params.min_score = minScore.value
  store.loadRuns(params)
}

function onSearchChange() {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(doLoad, 300)
}

function onScoreChange() {
  doLoad()
}

onMounted(() => {
  store.loadRuns({ limit: 50, offset: 0 })
})
</script>

<template>
  <div class="run-list-page">
    <div class="page-header">
      <h1>Agent Runs</h1>
      <div class="filters">
        <input
          v-model="search"
          type="text"
          placeholder="Search by task name..."
          class="filter-input"
          @input="onSearchChange"
        />
        <div class="score-filter">
          <label for="min-score">Min Score:</label>
          <input
            id="min-score"
            v-model.number="minScore"
            type="number"
            min="0"
            max="100"
            class="score-input"
            @input="onScoreChange"
          />
        </div>
      </div>
    </div>

    <div v-if="store.loading" class="loading">Loading runs...</div>

    <div v-else-if="store.runs.length === 0" class="empty-state">
      <p>No runs found.</p>
    </div>

    <div v-else class="runs-grid">
      <RunCard v-for="run in store.runs" :key="run.run_id" :run="run" />
    </div>

    <div v-if="store.total > 0" class="results-info">
      {{ store.runs.length }} of {{ store.total }} runs
    </div>
  </div>
</template>

<style scoped>
.run-list-page {
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
}
.page-header h1 {
  font-size: 22px;
  font-weight: 700;
  color: #e1e4e8;
}
.filters {
  display: flex;
  align-items: center;
  gap: 12px;
}
.filter-input {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 6px;
  padding: 8px 12px;
  color: #e1e4e8;
  font-size: 14px;
  min-width: 220px;
  outline: none;
  transition: border-color 0.15s;
}
.filter-input:focus {
  border-color: #58a6ff;
}
.filter-input::placeholder {
  color: #484f58;
}
.score-filter {
  display: flex;
  align-items: center;
  gap: 6px;
}
.score-filter label {
  font-size: 13px;
  color: #8b949e;
  white-space: nowrap;
}
.score-input {
  width: 64px;
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 6px;
  padding: 8px 10px;
  color: #e1e4e8;
  font-size: 14px;
  outline: none;
  transition: border-color 0.15s;
}
.score-input:focus {
  border-color: #58a6ff;
}
.loading {
  color: #8b949e;
  font-size: 16px;
  text-align: center;
  padding: 40px 0;
}
.empty-state {
  text-align: center;
  padding: 60px 0;
  color: #8b949e;
  font-size: 16px;
}
.runs-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}
.results-info {
  font-size: 13px;
  color: #484f58;
  text-align: right;
}
</style>
