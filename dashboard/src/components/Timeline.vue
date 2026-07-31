<script setup lang="ts">
import type { ToolCallRecord } from '@/types'
import { ref } from 'vue'

defineProps<{
  toolCalls: ToolCallRecord[]
  highlightIndexes?: number[]
}>()

const expanded = ref<Record<number, boolean>>({})

function toggle(idx: number) {
  expanded.value[idx] = !expanded.value[idx]
}

function formatArgs(args: Record<string, unknown>): string {
  try {
    return JSON.stringify(args, null, 2)
  } catch {
    return String(args)
  }
}

function formatTime(ts: string): string {
  const d = new Date(ts)
  return d.toLocaleTimeString()
}
</script>

<template>
  <div class="timeline">
    <div v-if="toolCalls.length === 0" class="empty">No tool calls recorded</div>
    <div
      v-for="(call, idx) in toolCalls"
      :key="idx"
      class="timeline-item"
    >
      <div class="timeline-dot" :class="{ blocked: call.blocked, error: call.is_error }" />
      <div
        class="timeline-content"
        :class="{ highlighted: highlightIndexes && highlightIndexes.includes(idx) }"
        @click="toggle(idx)"
      >
        <div class="timeline-header">
          <span class="tool-name">{{ call.tool_name }}</span>
          <div class="badges">
            <span v-if="call.blocked" class="badge badge-blocked">blocked</span>
            <span v-if="call.is_error" class="badge badge-error">error</span>
          </div>
          <span class="tool-time">{{ formatTime(call.timestamp) }}</span>
        </div>
        <div v-if="expanded[idx]" class="timeline-detail">
          <div class="detail-section">
            <strong>Arguments:</strong>
            <pre>{{ formatArgs(call.arguments) }}</pre>
          </div>
          <div class="detail-section">
            <strong>Result:</strong>
            <pre>{{ call.result }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.timeline {
  position: relative;
  padding-left: 24px;
}
.timeline::before {
  content: '';
  position: absolute;
  left: 7px;
  top: 0;
  bottom: 0;
  width: 2px;
  background: #30363d;
}
.empty {
  color: #8b949e;
  font-size: 14px;
  padding: 12px 0;
}
.timeline-item {
  position: relative;
  padding-bottom: 16px;
}
.timeline-dot {
  position: absolute;
  left: -21px;
  top: 4px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #30363d;
  border: 2px solid #484f58;
  z-index: 1;
}
.timeline-dot.blocked {
  background: #f85149;
  border-color: #f85149;
}
.timeline-dot.error {
  background: #f85149;
  border-color: #f85149;
}
.timeline-content {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 6px;
  padding: 10px 14px;
  cursor: pointer;
  transition: border-color 0.15s;
}
.timeline-content:hover {
  border-color: #58a6ff;
}
.timeline-content.highlighted {
  border-color: #d29922;
  background: rgba(210, 153, 34, 0.08);
  box-shadow: 0 0 0 1px rgba(210, 153, 34, 0.5);
}
.timeline-header {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.tool-name {
  font-weight: 600;
  font-size: 14px;
  color: #58a6ff;
}
.tool-time {
  margin-left: auto;
  font-size: 12px;
  color: #8b949e;
}
.badges {
  display: flex;
  gap: 4px;
}
.badge {
  font-size: 11px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 10px;
  text-transform: uppercase;
}
.badge-blocked {
  background: rgba(248, 81, 73, 0.15);
  color: #f85149;
}
.badge-error {
  background: rgba(248, 81, 73, 0.15);
  color: #f85149;
}
.timeline-detail {
  margin-top: 10px;
  border-top: 1px solid #30363d;
  padding-top: 10px;
}
.detail-section {
  margin-bottom: 8px;
}
.detail-section strong {
  display: block;
  font-size: 12px;
  color: #8b949e;
  margin-bottom: 4px;
}
.detail-section pre {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 4px;
  padding: 8px 10px;
  font-size: 12px;
  color: #e1e4e8;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
