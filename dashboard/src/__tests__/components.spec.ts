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

describe('SafetyScore edge cases', () => {
  it('labels exactly 80 as Safe', () => {
    const wrapper = mount(SafetyScore, { props: { score: 80 } })
    expect(wrapper.find('.score-label').text()).toBe('Safe')
  })

  it('labels 79 and 50 as Caution, 49 as Risk', () => {
    expect(mount(SafetyScore, { props: { score: 79 } }).find('.score-label').text()).toBe('Caution')
    expect(mount(SafetyScore, { props: { score: 50 } }).find('.score-label').text()).toBe('Caution')
    expect(mount(SafetyScore, { props: { score: 49 } }).find('.score-label').text()).toBe('Risk')
  })

  it('clamps out-of-range scores for label', () => {
    expect(mount(SafetyScore, { props: { score: 150 } }).find('.score-label').text()).toBe('Safe')
    expect(mount(SafetyScore, { props: { score: -10 } }).find('.score-label').text()).toBe('Risk')
  })

  it('picks ring color per band', () => {
    expect(mount(SafetyScore, { props: { score: 90 } }).find('.score-ring').attributes('stroke')).toBe('#3fb950')
    expect(mount(SafetyScore, { props: { score: 60 } }).find('.score-ring').attributes('stroke')).toBe('#d29922')
    expect(mount(SafetyScore, { props: { score: 20 } }).find('.score-ring').attributes('stroke')).toBe('#f85149')
  })
})

describe('Timeline highlighting', () => {
  const base: ToolCallRecord = {
    tool_name: 'bash',
    arguments: {},
    result: '',
    blocked: false,
    is_error: false,
    timestamp: '2026-07-31T12:00:00Z',
  }

  it('applies highlighted class only to indexed calls', () => {
    const calls = [base, { ...base, tool_name: 'read_file' }]
    const wrapper = mount(Timeline, {
      props: { toolCalls: calls, highlightIndexes: [1] },
    })
    const contents = wrapper.findAll('.timeline-content')
    expect(contents[0].classes()).not.toContain('highlighted')
    expect(contents[1].classes()).toContain('highlighted')
  })

  it('shows error badge for error calls', () => {
    const wrapper = mount(Timeline, {
      props: { toolCalls: [{ ...base, is_error: true }] },
    })
    expect(wrapper.find('.badge-error').text()).toBe('error')
  })
})

describe('RunCard state variants', () => {
  const baseRun: RunSummary = {
    run_id: 'r1',
    task_name: 't',
    created_at: '2026-07-31T10:00:00Z',
    model: 'm',
    status: 'failed',
    safety_score: 40,
    total_turns: 2,
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

  async function mountCardWith(run: RunSummary) {
    const router = makeRouter()
    const wrapper = mount(RunCard, { props: { run }, global: { plugins: [router] } })
    await router.push('/')
    await router.isReady()
    return wrapper
  }

  it('maps failed and blocked to badge-failed', async () => {
    for (const status of ['failed', 'blocked']) {
      const wrapper = await mountCardWith({ ...baseRun, status })
      expect(wrapper.find('.status-badge').classes()).toContain('badge-failed')
    }
  })

  it('maps timeout to badge-timeout', async () => {
    const wrapper = await mountCardWith({ ...baseRun, status: 'timeout' })
    expect(wrapper.find('.status-badge').classes()).toContain('badge-timeout')
  })

  it('applies score-risk chip for low score and score-safe for high score', async () => {
    const risk = await mountCardWith(baseRun)
    expect(risk.find('.score-chip').classes()).toContain('score-risk')

    const safe = await mountCardWith({ ...baseRun, safety_score: 90 })
    expect(safe.find('.score-chip').classes()).toContain('score-safe')
  })

  it('formats duration as seconds and minutes', async () => {
    const sec = await mountCardWith({ ...baseRun, total_duration_ms: 1500 })
    expect(sec.text()).toContain('1.5s')

    const min = await mountCardWith({ ...baseRun, total_duration_ms: 90000 })
    expect(min.text()).toContain('1m 30s')
  })
})
