<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { uploadRun } from '@/api/client'

const router = useRouter()

const taskName = ref('')
const model = ref('')
const systemPrompt = ref('')
const status = ref('success')
const maxTurns = ref(10)
const totalTurns = ref(0)
const totalTokens = ref(0)
const totalDurationMs = ref(0)
const eventsJson = ref('')

const loading = ref(false)
const error = ref('')
const fileName = ref('')

function handleFileUpload(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  fileName.value = file.name
  const reader = new FileReader()
  reader.onload = (ev) => {
    const text = ev.target?.result
    if (typeof text === 'string') {
      eventsJson.value = text
    }
  }
  reader.onerror = () => {
    error.value = 'Failed to read file.'
  }
  reader.readAsText(file)
}

function clearFile() {
  fileName.value = ''
  eventsJson.value = ''
  const input = document.getElementById('file-upload') as HTMLInputElement
  if (input) input.value = ''
}

async function submitForm() {
  error.value = ''

  if (!taskName.value.trim()) {
    error.value = 'Task name is required.'
    return
  }
  if (!model.value.trim()) {
    error.value = 'Model is required.'
    return
  }

  let parsed: string = eventsJson.value
  if (eventsJson.value.trim()) {
    try {
      const obj = JSON.parse(eventsJson.value)
      parsed = JSON.stringify(obj)
    } catch {
      error.value = 'Events JSON is not valid JSON.'
      return
    }
  }

  loading.value = true
  try {
    const run = await uploadRun({
      task_name: taskName.value.trim(),
      model: model.value.trim(),
      system_prompt: systemPrompt.value,
      status: status.value,
      max_turns: maxTurns.value,
      total_turns: totalTurns.value,
      total_tokens: totalTokens.value,
      total_duration_ms: totalDurationMs.value,
      events_json: parsed,
    })
    router.push(`/runs/${run.run_id}`)
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : 'Upload failed'
    error.value = msg
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="upload-page">
    <h1>Upload Run</h1>

    <form class="upload-form" @submit.prevent="submitForm">
      <div class="form-row">
        <div class="form-group">
          <label for="task-name">Task Name *</label>
          <input
            id="task-name"
            v-model="taskName"
            type="text"
            placeholder="e.g. customer_support_agent"
            class="form-input"
          />
        </div>
        <div class="form-group">
          <label for="model">Model *</label>
          <input
            id="model"
            v-model="model"
            type="text"
            placeholder="e.g. claude-sonnet-4-20250514"
            class="form-input"
          />
        </div>
      </div>

      <div class="form-group">
        <label for="system-prompt">System Prompt</label>
        <textarea
          id="system-prompt"
          v-model="systemPrompt"
          rows="3"
          placeholder="Enter system prompt..."
          class="form-input form-textarea"
        />
      </div>

      <div class="form-row">
        <div class="form-group">
          <label for="status">Status</label>
          <select id="status" v-model="status" class="form-input form-select">
            <option value="success">success</option>
            <option value="failed">failed</option>
            <option value="blocked">blocked</option>
            <option value="timeout">timeout</option>
          </select>
        </div>
        <div class="form-group">
          <label for="max-turns">Max Turns</label>
          <input
            id="max-turns"
            v-model.number="maxTurns"
            type="number"
            min="1"
            class="form-input"
          />
        </div>
      </div>

      <div class="form-row">
        <div class="form-group">
          <label for="total-turns">Total Turns</label>
          <input
            id="total-turns"
            v-model.number="totalTurns"
            type="number"
            min="0"
            class="form-input"
          />
        </div>
        <div class="form-group">
          <label for="total-tokens">Total Tokens</label>
          <input
            id="total-tokens"
            v-model.number="totalTokens"
            type="number"
            min="0"
            class="form-input"
          />
        </div>
        <div class="form-group">
          <label for="duration">Duration (ms)</label>
          <input
            id="duration"
            v-model.number="totalDurationMs"
            type="number"
            min="0"
            class="form-input"
          />
        </div>
      </div>

      <div class="form-group">
        <label for="events-json">Events JSON</label>
        <textarea
          id="events-json"
          v-model="eventsJson"
          rows="10"
          placeholder='Paste JSON array of events or upload a file...'
          class="form-input form-textarea form-textarea-lg"
        />
      </div>

      <div class="form-group">
        <label>Upload JSON File</label>
        <div class="file-upload-area">
          <input
            id="file-upload"
            type="file"
            accept=".json,.jsonl"
            class="file-input"
            @change="handleFileUpload"
          />
          <span v-if="fileName" class="file-name">
            {{ fileName }}
            <button type="button" class="clear-file-btn" @click="clearFile">&times;</button>
          </span>
          <span v-else class="file-placeholder">No file selected</span>
        </div>
      </div>

      <div v-if="error" class="error-msg">{{ error }}</div>

      <button type="submit" class="submit-btn" :disabled="loading">
        {{ loading ? 'Uploading...' : 'Submit Run' }}
      </button>
    </form>
  </div>
</template>

<style scoped>
.upload-page {
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.upload-page h1 {
  font-size: 22px;
  font-weight: 700;
  color: #e1e4e8;
}
.upload-form {
  display: flex;
  flex-direction: column;
  gap: 18px;
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 24px;
}
.form-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 16px;
}
.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.form-group label {
  font-size: 13px;
  font-weight: 600;
  color: #c9d1d9;
}
.form-input {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 6px;
  padding: 8px 12px;
  color: #e1e4e8;
  font-size: 14px;
  outline: none;
  transition: border-color 0.15s;
  font-family: inherit;
}
.form-input:focus {
  border-color: #58a6ff;
}
.form-input::placeholder {
  color: #484f58;
}
.form-textarea {
  resize: vertical;
}
.form-textarea-lg {
  font-family: monospace;
  font-size: 12px;
}
.form-select {
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%238b949e' d='M6 8L1 3h10z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  padding-right: 30px;
}
.file-upload-area {
  display: flex;
  align-items: center;
  gap: 10px;
}
.file-input {
  color: #c9d1d9;
  font-size: 13px;
}
.file-input::file-selector-button {
  background: #21262d;
  border: 1px solid #30363d;
  border-radius: 6px;
  color: #c9d1d9;
  padding: 6px 14px;
  cursor: pointer;
  margin-right: 10px;
}
.file-input::file-selector-button:hover {
  border-color: #58a6ff;
}
.file-name {
  font-size: 13px;
  color: #58a6ff;
  display: flex;
  align-items: center;
  gap: 6px;
}
.clear-file-btn {
  background: none;
  border: none;
  color: #8b949e;
  font-size: 18px;
  cursor: pointer;
  line-height: 1;
}
.clear-file-btn:hover {
  color: #f85149;
}
.file-placeholder {
  font-size: 13px;
  color: #484f58;
}
.error-msg {
  background: rgba(248, 81, 73, 0.1);
  border: 1px solid rgba(248, 81, 73, 0.3);
  border-radius: 6px;
  padding: 10px 14px;
  color: #f85149;
  font-size: 14px;
}
.submit-btn {
  background: #238636;
  border: 1px solid rgba(240, 246, 252, 0.1);
  border-radius: 6px;
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  padding: 10px 24px;
  cursor: pointer;
  transition: background 0.15s;
  align-self: flex-start;
}
.submit-btn:hover:not(:disabled) {
  background: #2ea043;
}
.submit-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
