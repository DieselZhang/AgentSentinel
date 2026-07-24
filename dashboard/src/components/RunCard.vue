<script setup lang="ts">
import type { RunSummary } from '@/types'
import { computed } from 'vue'

const props = defineProps<{
  run: RunSummary
}>()

const statusClass = computed(() => {
  switch (props.run.status) {
    case 'success': return 'badge-success'
    case 'failed':
    case 'blocked': return 'badge-failed'
    case 'timeout': return 'badge-timeout'
    default: return 'badge-default'
  }
})

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`
  const mins = Math.floor(ms / 60000)
  const secs = Math.round((ms % 60000) / 1000)
  return `${mins}m ${secs}s`
}

function formatDate(d: string): string {
  return new Date(d).toLocaleDateString()
}
</script>

<template>
  <router-link :to="`/runs/${run.run_id}`" class="run-card">
    <div class="card-top">
      <span class="task-name">{{ run.task_name }}</span>
      <span class="status-badge" :class="statusClass">{{ run.status }}</span>
    </div>
    <div class="card-meta">
      <span class="meta-item">{{ run.model }}</span>
      <span class="meta-item">{{ run.total_turns }} turns</span>
      <span class="meta-item">{{ formatDuration(run.total_duration_ms) }}</span>
    </div>
    <div class="card-footer">
      <span class="date">{{ formatDate(run.created_at) }}</span>
      <span class="score-chip" :class="{
        'score-safe': run.safety_score >= 80,
        'score-caution': run.safety_score >= 50 && run.safety_score < 80,
        'score-risk': run.safety_score < 50,
      }">
        {{ run.safety_score }}
      </span>
    </div>
  </router-link>
</template>

<style scoped>
.run-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 16px;
  text-decoration: none;
  color: inherit;
  transition: border-color 0.15s, transform 0.1s;
}
.run-card:hover {
  border-color: #58a6ff;
  transform: translateY(-1px);
}
.card-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.task-name {
  font-size: 15px;
  font-weight: 600;
  color: #e1e4e8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.status-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 10px;
  text-transform: uppercase;
  letter-spacing: 0.3px;
  flex-shrink: 0;
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
.badge-default {
  background: rgba(139, 148, 158, 0.15);
  color: #8b949e;
}
.card-meta {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}
.meta-item {
  font-size: 12px;
  color: #8b949e;
}
.card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-top: 1px solid #30363d;
  padding-top: 10px;
}
.date {
  font-size: 12px;
  color: #484f58;
}
.score-chip {
  font-size: 14px;
  font-weight: 700;
  min-width: 36px;
  text-align: center;
  padding: 2px 8px;
  border-radius: 6px;
}
.score-safe {
  background: rgba(63, 185, 80, 0.15);
  color: #3fb950;
}
.score-caution {
  background: rgba(210, 153, 34, 0.15);
  color: #d29922;
}
.score-risk {
  background: rgba(248, 81, 73, 0.15);
  color: #f85149;
}
</style>
