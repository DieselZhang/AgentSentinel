<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  score: number
  size?: number
}>(), {
  size: 120,
})

const radius = computed(() => 52)
const circumference = computed(() => 2 * Math.PI * radius.value)
const offset = computed(() => {
  const pct = Math.min(100, Math.max(0, props.score)) / 100
  return circumference.value * (1 - pct)
})

const color = computed(() => {
  if (props.score >= 80) return '#3fb950'
  if (props.score >= 50) return '#d29922'
  return '#f85149'
})

const label = computed(() => {
  if (props.score >= 80) return 'Safe'
  if (props.score >= 50) return 'Caution'
  return 'Risk'
})
</script>

<template>
  <div class="safety-score" :style="{ width: size + 'px', height: size + 'px' }">
    <svg :width="size" :height="size" viewBox="0 0 120 120">
      <circle
        cx="60"
        cy="60"
        :r="radius"
        fill="none"
        stroke="#21262d"
        stroke-width="8"
      />
      <circle
        cx="60"
        cy="60"
        :r="radius"
        fill="none"
        :stroke="color"
        stroke-width="8"
        stroke-linecap="round"
        :stroke-dasharray="circumference"
        :stroke-dashoffset="offset"
        transform="rotate(-90 60 60)"
        class="score-ring"
      />
    </svg>
    <div class="score-inner">
      <span class="score-value" :style="{ color }">{{ score }}</span>
      <span class="score-label" :style="{ color }">{{ label }}</span>
    </div>
  </div>
</template>

<style scoped>
.safety-score {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.safety-score svg {
  position: absolute;
  top: 0;
  left: 0;
}
.score-ring {
  transition: stroke-dashoffset 0.6s ease;
}
.score-inner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}
.score-value {
  font-size: 28px;
  font-weight: 700;
  line-height: 1;
}
.score-label {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
</style>
