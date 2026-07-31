import { describe, it, expect } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createRouter, createMemoryHistory, type Router } from 'vue-router'
import SafetyScore from '@/components/SafetyScore.vue'
import Timeline from '@/components/Timeline.vue'
import RunCard from '@/components/RunCard.vue'
import type { ToolCallRecord, RunSummary } from '@/types'

describe('SafetyScore', () => {
  it('renders the score value', () => {
    const wrapper = mount(SafetyScore, { props: { score: 85 } })
    expect(wrapper.find('.score-value').text()).toBe('85')
  })

  it('shows Safe label for score >= 80', () => {
    const wrapper = mount(SafetyScore, { props: { score: 92 } })
    expect(wrapper.find('.score-label').text()).toBe('Safe')
  })

  it('shows Caution label for 50 <= score < 80', () => {
    const wrapper = mount(SafetyScore, { props: { score: 65 } })
    expect(wrapper.find('.score-label').text()).toBe('Caution')
  })

  it('shows Risk label for score < 50', () => {
    const wrapper = mount(SafetyScore, { props: { score: 30 } })
    expect(wrapper.find('.score-label').text()).toBe('Risk')
  })
})

describe('Timeline', () => {
  const toolCall: ToolCallRecord = {
    tool_name: 'web_search',
    arguments: { query: 'test' },
    result: 'found results',
    blocked: false,
    is_error: false,
    timestamp: '2026-07-31T12:00:00Z',
  }

  it('shows empty message when no tool calls', () => {
    const wrapper = mount(Timeline, { props: { toolCalls: [] } })
    expect(wrapper.text()).toContain('No tool calls recorded')
  })

  it('renders tool names for each call', () => {
    const wrapper = mount(Timeline, { props: { toolCalls: [toolCall] } })
    expect(wrapper.find('.tool-name').text()).toBe('web_search')
  })

  it('shows blocked badge for blocked calls', () => {
    const wrapper = mount(Timeline, {
      props: { toolCalls: [{ ...toolCall, blocked: true }] },
    })
    expect(wrapper.find('.badge-blocked').text()).toBe('blocked')
  })

  it('expands on click to reveal arguments and result', async () => {
    const wrapper = mount(Timeline, { props: { toolCalls: [toolCall] } })
    expect(wrapper.find('.timeline-detail').exists()).toBe(false)

    await wrapper.find('.timeline-content').trigger('click')

    expect(wrapper.find('.timeline-detail').exists()).toBe(true)
    expect(wrapper.text()).toContain('Arguments:')
    expect(wrapper.text()).toContain('Result:')
    expect(wrapper.text()).toContain('found results')
    expect(wrapper.text()).toContain('"query": "test"')
  })
})

describe('RunCard', () => {
  const run: RunSummary = {
    run_id: 'run-123',
    task_name: 'Test task',
    created_at: '2026-07-31T10:00:00Z',
    model: 'gpt-4o',
    status: 'success',
    safety_score: 95,
    total_turns: 5,
    total_duration_ms: 1500,
  }

  function makeRouter(): Router {
    return createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/', name: 'home', component: { template: '<div />' } },
        { path: '/runs/:id', name: 'run-detail', component: { template: '<div />' } },
      ],
    })
  }

  async function mountCard(router: Router) {
    const wrapper = mount(RunCard, {
      props: { run },
      global: { plugins: [router] },
    })
    await router.push('/')
    await router.isReady()
    return wrapper
  }

  it('renders task name, model, status and safety score', async () => {
    const wrapper = await mountCard(makeRouter())
    expect(wrapper.find('.task-name').text()).toBe('Test task')
    expect(wrapper.text()).toContain('gpt-4o')
    expect(wrapper.find('.status-badge').text()).toBe('success')
    expect(wrapper.find('.score-chip').text()).toBe('95')
  })

  it('applies success badge class for success status', async () => {
    const wrapper = await mountCard(makeRouter())
    expect(wrapper.find('.status-badge').classes()).toContain('badge-success')
  })

  it('links to /runs/{run_id}', async () => {
    const wrapper = await mountCard(makeRouter())
    expect(wrapper.find('.run-card').attributes('href')).toBe('/runs/run-123')
  })

  it('navigates to /runs/{run_id} on click', async () => {
    const router = makeRouter()
    const wrapper = await mountCard(router)
    await wrapper.find('.run-card').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/runs/run-123')
  })
})
