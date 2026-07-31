<script setup lang="ts">
import { ref } from 'vue'
import { useRunsStore } from '@/stores/runs'
import SafetyScore from '@/components/SafetyScore.vue'
import Timeline from '@/components/Timeline.vue'
import type { RunDetail } from '@/types'

const store = useRunsStore()

const id1 = ref('')
const id2 = ref('')
const error = ref('')

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`
  const mins = Math.floor(ms / 60000)
  const secs = Math.round((ms % 60000) / 1000)
  return `${mins}m ${secs}s`
}

function diffText(v: number, formatNumber = false): string {
  if (v === 0) return '='
  const sign = v > 0 ? '+' : ''
  return sign + (formatNumber ? v.toLocaleString() : String(v))
}

function diffClass(v: number): string {
  if (v === 0) return 'diff-zero'
  return v > 0 ? 'diff-pos' : 'diff-neg'
}

function formatSignedDuration(ms: number): string {
  if (ms === 0) return '='
  return `${ms > 0 ? '+' : '-'}${formatDuration(Math.abs(ms))}`
}

function otherRun(runIdx: number): RunDetail | null {
  if (store.comparedRuns.length !== 2) return null
  return store.comparedRuns[runIdx === 0 ? 1 : 0]
}

function isHigherScore(runIdx: number): boolean {
  const run = store.comparedRuns[runIdx]
  const other = otherRun(runIdx)
  if (!run || !other) return false
  return run.safety_score > other.safety_score
}

function differingToolIndexes(runIdx: number): number[] {
  const run = store.comparedRuns[runIdx]
  const other = otherRun(runIdx)
  if (!run || !other) return []
  const indexes: number[] = []
  run.tool_calls.forEach((call, idx) => {
    const differs = other.tool_calls.some(
      (oc) =>
        oc.tool_name === call.tool_name &&
        (oc.blocked !== call.blocked || oc.result !== call.result),
    )
    if (differs) indexes.push(idx)
  })
  return indexes
}

async function doCompare() {
  error.value = ''
  const trimmed1 = id1.value.trim()
  const trimmed2 = id2.value.trim()
  if (!trimmed1 || !trimmed2) {
    error.value = 'Please enter both run IDs.'
    return
  }
  if (trimmed1 === trimmed2) {
    error.value = 'Please enter two different run IDs.'
    return
  }
  await store.loadCompare([trimmed1, trimmed2])
}
</script>

<template>
  <div class="compare-page">
    <h1>Compare Runs</h1>

    <div class="compare-inputs">
      <div class="input-group">
        <label for="run-id-1">Run ID 1</label>
        <input
          id="run-id-1"
          v-model="id1"
          type="text"
          placeholder="Enter run ID..."
          class="run-input"
          @keyup.enter="doCompare"
        />
      </div>
      <div class="input-group">
        <label for="run-id-2">Run ID 2</label>
        <input
          id="run-id-2"
          v-model="id2"
          type="text"
          placeholder="Enter run ID..."
          class="run-input"
          @keyup.enter="doCompare"
        />
      </div>
      <button class="compare-btn" :disabled="store.loading" @click="doCompare">
        {{ store.loading ? 'Loading...' : 'Compare' }}
      </button>
    </div>

    <div v-if="error" class="error-msg">{{ error }}</div>

    <div v-if="store.loading" class="loading">Loading comparison...</div>

    <template v-else-if="store.comparedRuns.length === 2">
      <div v-if="store.comparison" class="diff-card">
        <h3>Differences (Run 1 vs Run 2)</h3>
        <div class="diff-grid">
          <div class="diff-item">
            <span class="diff-key">Safety Score</span>
            <span class="diff-val" :class="diffClass(store.comparison.score_diff)">
              {{ diffText(store.comparison.score_diff) }}
            </span>
          </div>
          <div class="diff-item">
            <span class="diff-key">Turns</span>
            <span class="diff-val" :class="diffClass(store.comparison.turns_diff)">
              {{ diffText(store.comparison.turns_diff) }}
            </span>
          </div>
          <div class="diff-item">
            <span class="diff-key">Tokens</span>
            <span class="diff-val" :class="diffClass(store.comparison.tokens_diff)">
              {{ diffText(store.comparison.tokens_diff, true) }}
            </span>
          </div>
          <div class="diff-item">
            <span class="diff-key">Duration</span>
            <span class="diff-val" :class="diffClass(store.comparison.duration_diff_ms)">
              {{ formatSignedDuration(store.comparison.duration_diff_ms) }}
            </span>
          </div>
        </div>
      </div>

      <div class="compare-results">
      <div
        v-for="(run, idx) in store.comparedRuns"
        :key="run.run_id"
        class="compare-column"
      >
        <div class="column-header">
          <h2>{{ run.task_name }}</h2>
          <span class="run-id-label">ID: {{ run.run_id }}</span>
        </div>

        <div class="score-center">
          <div class="score-badge-wrap" :class="{ 'score-higher': isHigherScore(idx) }">
            <SafetyScore :score="run.safety_score" :size="140" />
          </div>
        </div>

        <div class="stats-table">
          <div class="stat-row">
            <span class="stat-key">Status</span>
            <span class="stat-val">
              <span class="status-badge" :class="{
                'badge-success': run.status === 'success',
                'badge-failed': run.status === 'failed' || run.status === 'blocked',
                'badge-timeout': run.status === 'timeout',
              }">{{ run.status }}</span>
            </span>
          </div>
          <div class="stat-row">
            <span class="stat-key">Model</span>
            <span class="stat-val">{{ run.model }}</span>
          </div>
          <div class="stat-row">
            <span class="stat-key">Turns</span>
            <span class="stat-val">{{ run.total_turns }} / {{ run.max_turns }}</span>
          </div>
          <div class="stat-row">
            <span class="stat-key">Tokens</span>
            <span class="stat-val">{{ run.total_tokens.toLocaleString() }}</span>
          </div>
          <div class="stat-row">
            <span class="stat-key">Duration</span>
            <span class="stat-val">{{ formatDuration(run.total_duration_ms) }}</span>
          </div>
          <div class="stat-row">
            <span class="stat-key">Alerts</span>
            <span class="stat-val">{{ run.alerts.length }}</span>
          </div>
          <div class="stat-row">
            <span class="stat-key">Tool Calls</span>
            <span class="stat-val">{{ run.tool_calls.length }}</span>
          </div>
        </div>

        <div class="compare-section">
          <h3>Tool Calls</h3>
          <Timeline :tool-calls="run.tool_calls" :highlight-indexes="differingToolIndexes(idx)" />
        </div>
      </div>
    </div>
    </template>
  </div>
</template>

<style scoped>
.compare-page {
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.compare-page h1 {
  font-size: 22px;
  font-weight: 700;
  color: #e1e4e8;
}
.compare-inputs {
  display: flex;
  align-items: flex-end;
  gap: 12px;
  flex-wrap: wrap;
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 16px;
}
.input-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.input-group label {
  font-size: 12px;
  color: #8b949e;
  font-weight: 600;
}
.run-input {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 6px;
  padding: 8px 12px;
  color: #e1e4e8;
  font-size: 14px;
  min-width: 240px;
  outline: none;
  transition: border-color 0.15s;
}
.run-input:focus {
  border-color: #58a6ff;
}
.run-input::placeholder {
  color: #484f58;
}
.compare-btn {
  background: #238636;
  border: 1px solid rgba(240, 246, 252, 0.1);
  border-radius: 6px;
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  padding: 8px 20px;
  cursor: pointer;
  transition: background 0.15s;
  height: fit-content;
}
.compare-btn:hover:not(:disabled) {
  background: #2ea043;
}
.compare-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.error-msg {
  background: rgba(248, 81, 73, 0.1);
  border: 1px solid rgba(248, 81, 73, 0.3);
  border-radius: 6px;
  padding: 10px 14px;
  color: #f85149;
  font-size: 14px;
}
.loading {
  text-align: center;
  padding: 40px 0;
  color: #8b949e;
  font-size: 16px;
}
.diff-card {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 16px 20px;
}
.diff-card h3 {
  font-size: 14px;
  font-weight: 600;
  color: #e1e4e8;
  margin-bottom: 12px;
}
.diff-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}
.diff-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 6px;
  padding: 10px 12px;
}
.diff-key {
  font-size: 12px;
  color: #8b949e;
  font-weight: 600;
}
.diff-val {
  font-size: 18px;
  font-weight: 700;
}
.diff-pos {
  color: #3fb950;
}
.diff-neg {
  color: #f85149;
}
.diff-zero {
  color: #8b949e;
}
.compare-results {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}
.compare-column {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.column-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: #e1e4e8;
}
.run-id-label {
  font-size: 12px;
  color: #484f58;
  font-family: monospace;
  margin-top: 4px;
  display: block;
}
.score-center {
  display: flex;
  justify-content: center;
  padding: 12px 0;
}
.score-badge-wrap {
  border: 2px solid transparent;
  border-radius: 50%;
  padding: 4px;
  transition: border-color 0.2s;
}
.score-badge-wrap.score-higher {
  border-color: #3fb950;
}
.stats-table {
  display: flex;
  flex-direction: column;
  gap: 0;
  border: 1px solid #30363d;
  border-radius: 6px;
  overflow: hidden;
}
.stat-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  border-bottom: 1px solid #30363d;
  background: #0d1117;
}
.stat-row:last-child {
  border-bottom: none;
}
.stat-key {
  font-size: 13px;
  color: #8b949e;
}
.stat-val {
  font-size: 14px;
  font-weight: 600;
  color: #e1e4e8;
}
.status-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 10px;
  text-transform: uppercase;
}
.badge-success {
  background: rgba(63, 185, 80, 0.15);
  color: #3fb950;
}
.badge-failed {
  background: rgba(248, 81, 73, 0.15);
  color: #f85149;
}
.badge-timeout {
  background: rgba(88, 166, 255, 0.15);
  color: #58a6ff;
}
.compare-section {
  margin-top: 4px;
}
.compare-section h3 {
  font-size: 14px;
  font-weight: 600;
  color: #e1e4e8;
  margin-bottom: 12px;
}

@media (max-width: 768px) {
  .compare-results {
    grid-template-columns: 1fr;
  }
  .diff-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
