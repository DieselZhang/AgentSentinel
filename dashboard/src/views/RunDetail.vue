<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { useRunsStore } from '@/stores/runs'
import SafetyScore from '@/components/SafetyScore.vue'
import Timeline from '@/components/Timeline.vue'

const route = useRoute()
const store = useRunsStore()

const showEventsJson = ref(false)

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`
  const mins = Math.floor(ms / 60000)
  const secs = Math.round((ms % 60000) / 1000)
  return `${mins}m ${secs}s`
}

function severityClass(severity: string): string {
  switch (severity) {
    case 'critical': return 'alert-critical'
    case 'high': return 'alert-high'
    case 'medium': return 'alert-medium'
    case 'low': return 'alert-low'
    default: return ''
  }
}

onMounted(() => {
  const id = route.params.id as string
  if (id) store.loadRun(id)
})
</script>

<template>
  <div class="run-detail-page">
    <div v-if="store.loading" class="loading">Loading run details...</div>

    <template v-else-if="store.currentRun">
      <div class="detail-header">
        <router-link to="/" class="back-link">&larr; Back to runs</router-link>
        <h1>{{ store.currentRun.task_name }}</h1>
      </div>

      <div class="score-section">
        <SafetyScore :score="store.currentRun.safety_score" :size="160" />
      </div>

      <div class="meta-grid">
        <div class="meta-card">
          <span class="meta-label">Status</span>
          <span class="meta-value">
            <span class="status-badge" :class="{
              'badge-success': store.currentRun.status === 'success',
              'badge-failed': store.currentRun.status === 'failed' || store.currentRun.status === 'blocked',
              'badge-timeout': store.currentRun.status === 'timeout',
            }">{{ store.currentRun.status }}</span>
          </span>
        </div>
        <div class="meta-card">
          <span class="meta-label">Model</span>
          <span class="meta-value">{{ store.currentRun.model }}</span>
        </div>
        <div class="meta-card">
          <span class="meta-label">Turns</span>
          <span class="meta-value">{{ store.currentRun.total_turns }} / {{ store.currentRun.max_turns }}</span>
        </div>
        <div class="meta-card">
          <span class="meta-label">Tokens</span>
          <span class="meta-value">{{ store.currentRun.total_tokens.toLocaleString() }}</span>
        </div>
        <div class="meta-card">
          <span class="meta-label">Duration</span>
          <span class="meta-value">{{ formatDuration(store.currentRun.total_duration_ms) }}</span>
        </div>
        <div class="meta-card">
          <span class="meta-label">Safety Score</span>
          <span class="meta-value">{{ store.currentRun.safety_score }}</span>
        </div>
      </div>

      <div v-if="store.currentRun.system_prompt" class="section">
        <h2>System Prompt</h2>
        <div class="prompt-box">{{ store.currentRun.system_prompt }}</div>
      </div>

      <div v-if="store.currentRun.alerts.length > 0" class="section">
        <h2>Safety Alerts ({{ store.currentRun.alerts.length }})</h2>
        <div
          v-for="(alert, idx) in store.currentRun.alerts"
          :key="idx"
          class="alert-item"
          :class="severityClass(alert.severity)"
        >
          <span class="alert-severity">{{ alert.severity }}</span>
          <span class="alert-message">{{ alert.message }}</span>
          <span class="alert-event">event #{{ alert.event_index }}</span>
        </div>
      </div>
      <div v-else class="section">
        <h2>Safety Alerts</h2>
        <p class="no-alerts">No safety alerts</p>
      </div>

      <div class="section">
        <h2>Tool Calls ({{ store.currentRun.tool_calls.length }})</h2>
        <Timeline :tool-calls="store.currentRun.tool_calls" />
      </div>

      <div class="section">
        <button class="toggle-btn" @click="showEventsJson = !showEventsJson">
          {{ showEventsJson ? 'Hide' : 'Show' }} Raw Events JSON
        </button>
        <pre v-if="showEventsJson" class="events-json">{{ store.currentRun.events_json }}</pre>
      </div>
    </template>

    <div v-else class="loading">Run not found.</div>
  </div>
</template>

<style scoped>
.run-detail-page {
  display: flex;
  flex-direction: column;
  gap: 24px;
}
.loading {
  text-align: center;
  padding: 40px 0;
  color: #8b949e;
  font-size: 16px;
}
.detail-header {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.back-link {
  font-size: 14px;
  color: #58a6ff;
  text-decoration: none;
}
.back-link:hover {
  text-decoration: underline;
}
.detail-header h1 {
  font-size: 24px;
  font-weight: 700;
  color: #e1e4e8;
}
.score-section {
  display: flex;
  justify-content: center;
  padding: 20px 0;
}
.meta-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 12px;
}
.meta-card {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.meta-label {
  font-size: 12px;
  color: #8b949e;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.meta-value {
  font-size: 16px;
  font-weight: 600;
  color: #e1e4e8;
}
.status-badge {
  font-size: 12px;
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
.section {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 20px;
}
.section h2 {
  font-size: 16px;
  font-weight: 600;
  color: #e1e4e8;
  margin-bottom: 14px;
}
.prompt-box {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 6px;
  padding: 12px 14px;
  font-size: 13px;
  color: #c9d1d9;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 200px;
  overflow-y: auto;
}
.no-alerts {
  font-size: 14px;
  color: #8b949e;
}
.alert-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 6px;
  margin-bottom: 8px;
  font-size: 13px;
}
.alert-item:last-child {
  margin-bottom: 0;
}
.alert-critical {
  background: rgba(248, 81, 73, 0.1);
  border: 1px solid rgba(248, 81, 73, 0.3);
}
.alert-high {
  background: rgba(248, 81, 73, 0.07);
  border: 1px solid rgba(248, 81, 73, 0.2);
}
.alert-medium {
  background: rgba(210, 153, 34, 0.1);
  border: 1px solid rgba(210, 153, 34, 0.3);
}
.alert-low {
  background: rgba(88, 166, 255, 0.07);
  border: 1px solid rgba(88, 166, 255, 0.2);
}
.alert-severity {
  font-weight: 600;
  text-transform: uppercase;
  font-size: 11px;
  min-width: 60px;
}
.alert-message {
  flex: 1;
  color: #e1e4e8;
}
.alert-event {
  font-size: 11px;
  color: #8b949e;
  flex-shrink: 0;
}
.toggle-btn {
  background: #21262d;
  border: 1px solid #30363d;
  border-radius: 6px;
  color: #c9d1d9;
  font-size: 13px;
  padding: 8px 14px;
  cursor: pointer;
  transition: border-color 0.15s;
}
.toggle-btn:hover {
  border-color: #58a6ff;
}
.events-json {
  margin-top: 12px;
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 6px;
  padding: 14px;
  font-size: 12px;
  color: #c9d1d9;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 400px;
  overflow-y: auto;
}
</style>
