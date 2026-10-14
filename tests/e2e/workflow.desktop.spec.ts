import { test, expect } from '@playwright/test'
import * as fs from 'node:fs'
import * as path from 'node:path'

test.describe.serial('Desktop Workflow Execution & Workspace Flow', () => {
  const stateDir = path.resolve(process.cwd(), 'tests/e2e/.state')
  const runStateFile = path.join(stateDir, 'active_run.json')

  let runId = ''
  let workspaceId = ''

  test('1. Create interview workspace and resolve associated static runId', async ({ page }) => {
    await page.goto('/interviews/new')
    await page.fill('#company-name', 'Cloudflare')
    await page.fill('#role-title', 'Senior Systems Engineer')
    await page.getByRole('button', { name: 'senior', exact: true }).click()
    await page.locator('[data-testid="wizard-continue"]').click()

    await page.locator('[data-testid="urls-text"]').fill('https://blog.cloudflare.com/')
    await page.locator('[data-testid="wizard-continue"]').click()

    await page.locator('[data-testid="resume-text"]').fill(
      'Senior Systems Engineer with 8 years of experience designing, deploying, and operating high-throughput distributed edge proxies, anycast networks, and Linux networking software. Proficient in Rust, C, Go, and eBPF. Experienced in zero-trust architectures, packet inspection pipelines, and distributed state caching across multi-region edge nodes.'
    )
    await page.locator('[data-testid="wizard-continue"]').click()

    await page.locator('[data-testid="jd-text"]').fill(
      'We are looking for a Senior Systems Engineer to build and scale global edge proxy infrastructure. In this role, you will design distributed network services handling millions of requests per second, optimize kernel networking and eBPF programs, enhance multi-region fault tolerance, and ensure sub-millisecond edge routing. You should have strong systems programming experience in Rust, C, or Go, deep understanding of TCP/IP, HTTP/3, and distributed consensus mechanisms, along with experience managing high-availability production fleets.'
    )
    await page.locator('[data-testid="wizard-submit"]').click()

    await page.waitForURL(/\/interviews\/([^/]+)\/overview/)
    const match = page.url().match(/\/interviews\/([^/]+)\/overview/)
    expect(match).not.toBeNull()
    workspaceId = match![1]

    await expect.poll(async () => {
      const res = await page.request.get(`/api/backend/workflows/runs?workspace_id=${workspaceId}`)
      if (!res.ok()) {
        throw new Error(`Failed to list runs for workspace: status ${res.status()} ${await res.text()}`)
      }
      const data = await res.json()
      return data.items?.length || 0
    }, { timeout: 120000, intervals: [2000, 5000] }).toBeGreaterThanOrEqual(1)

    const runsRes = await page.request.get(`/api/backend/workflows/runs?workspace_id=${workspaceId}`)
    if (!runsRes.ok()) {
      throw new Error(`Failed to fetch created run: status ${runsRes.status()} ${await runsRes.text()}`)
    }
    const runsData = await runsRes.json()
    expect(runsData.items?.length).toBeGreaterThanOrEqual(1)
    runId = runsData.items[0].id
    fs.mkdirSync(stateDir, { recursive: true })
    fs.writeFileSync(runStateFile, JSON.stringify({ runId, workspaceId }), 'utf8')
  })

  test('2. Mid-run reload preserves sequence continuity without duplicates', async ({ page }) => {
    await page.goto(`/workflows/runs/${runId}`)

    const taskNode = page.locator('[data-testid="task-node"]').first()
    await expect(taskNode).toBeVisible({ timeout: 120000 })

    await page.getByRole('button', { name: /Activity Timeline/i }).click()
    const timelineEvents = page.locator('[data-testid="timeline-event"]')
    await expect(timelineEvents.first()).toBeVisible({ timeout: 60000 })

    const snapshotRes = await page.request.get(`/api/backend/workflows/runs/${runId}`)
    if (!snapshotRes.ok()) {
      throw new Error(`Failed to get run snapshot: ${snapshotRes.status()} ${await snapshotRes.text()}`)
    }
    const initialRun = await snapshotRes.json()

    await page.reload()

    await expect(timelineEvents.first()).toBeVisible({ timeout: 60000 })
    const count = await timelineEvents.count()
    const seenSeqs = new Set<string>()

    for (let i = 0; i < count; i++) {
      const seqAttr = await timelineEvents.nth(i).getAttribute('data-seq')
      if (seqAttr) {
        expect(seenSeqs.has(seqAttr)).toBe(false)
        seenSeqs.add(seqAttr)
      }
    }

    const reloadedRes = await page.request.get(`/api/backend/workflows/runs/${runId}`)
    if (!reloadedRes.ok()) {
      throw new Error(`Failed to reload run snapshot: ${reloadedRes.status()} ${await reloadedRes.text()}`)
    }
    const reloadedRun = await reloadedRes.json()
    expect(reloadedRun.last_seq).toBeGreaterThanOrEqual(initialRun.last_seq)
  })

  test('3. Verify static run completion and pairwise-overlapping parallel intervals', async ({ page }) => {
    test.setTimeout(1_560_000)
    let runData: Record<string, unknown> = {}
    await expect.poll(async () => {
      const res = await page.request.get(`/api/backend/workflows/runs/${runId}`)
      if (!res.ok()) {
        throw new Error(`Run polling failed: status ${res.status()} ${await res.text()}`)
      }
      const data = (await res.json()) as Record<string, unknown>
      runData = data
      return data.status
    }, { timeout: 1_500_000, intervals: [3000, 6000] }).toBe('completed')

    if (runData.degraded) {
      const reasons = Array.isArray(runData.degraded_reasons) ? runData.degraded_reasons.join(', ') : 'Unknown degradation'
      throw new Error(`Run completed in degraded state: ${reasons}`)
    }

    const progress = (runData.progress || {}) as Record<string, unknown>
    expect(progress.failed ?? 0).toBe(0)

    const graphRes = await page.request.get(`/api/backend/workflows/runs/${runId}/graph`)
    if (!graphRes.ok()) {
      throw new Error(`Failed to fetch task graph: status ${graphRes.status()} ${await graphRes.text()}`)
    }
    const graph = (await graphRes.json()) as { nodes?: Array<Record<string, unknown>> }
    const taskNodes = graph.nodes || []
    expect(taskNodes.length).toBeGreaterThan(0)
    for (const node of taskNodes) {
      const nodeData = (node.data || {}) as Record<string, unknown>
      const nodeStatus = nodeData.status
      if (nodeStatus === 'skipped') {
        expect(nodeData.skip_reason || nodeData.reason || nodeData.error).toBeDefined()
      } else {
        expect(nodeStatus).toBe('completed')
      }
    }

    const postAgentsRes = await page.request.get(`/api/backend/workflows/runs/${runId}/agents`)
    if (!postAgentsRes.ok()) {
      throw new Error(`Failed to fetch agents post-completion: status ${postAgentsRes.status()}`)
    }
    const postAgents = ((await postAgentsRes.json()) || []) as Array<Record<string, unknown>>
    for (const a of postAgents) {
      expect(a.status).not.toBe('running')
      expect(a.status).not.toBe('pending')
    }

    const postDelsRes = await page.request.get(`/api/backend/workflows/runs/${runId}/delegations`)
    if (!postDelsRes.ok()) {
      throw new Error(`Failed to fetch delegations post-completion: status ${postDelsRes.status()}`)
    }
    const postDels = ((await postDelsRes.json()) || []) as Array<Record<string, unknown>>
    for (const d of postDels) {
      expect(['completed', 'failed', 'cancelled', 'rejected']).toContain(d.status)
    }
    const agentsRes = await page.request.get(`/api/backend/workflows/runs/${runId}/agents`)
    if (!agentsRes.ok()) {
      throw new Error(`Failed to fetch agents: status ${agentsRes.status()} ${await agentsRes.text()}`)
    }
    const agents = await agentsRes.json()
    expect(Array.isArray(agents)).toBe(true)
    expect(agents.length).toBeGreaterThanOrEqual(3)

    const intervals = agents
      .map((a: any) => ({
        id: a.id,
        start: new Date(a.started_at).getTime(),
        end: new Date(a.completed_at || a.started_at).getTime(),
      }))
      .filter((i: any) => !isNaN(i.start) && !isNaN(i.end))

    let overlappingPairs = 0
    for (let i = 0; i < intervals.length; i++) {
      for (let j = i + 1; j < intervals.length; j++) {
        const a = intervals[i]
        const b = intervals[j]
        if (Math.max(a.start, b.start) <= Math.min(a.end, b.end)) {
          overlappingPairs++
        }
      }
    }
    expect(overlappingPairs).toBeGreaterThanOrEqual(2)
  })

  test('4. Inspect agent execution, tool invocations, and sanitized inputs', async ({ page }) => {
    await page.goto(`/workflows/runs/${runId}`)

    const taskNode = page.locator('[data-testid="task-node"]').first()
    await expect(taskNode).toBeVisible({ timeout: 60000 })
    await taskNode.click()

    const inspector = page.locator('aside')
    await expect(inspector).toBeVisible()

    const toolRow = inspector.locator('[data-testid="tool-invocation-row"]').first()
    await expect(toolRow).toBeVisible({ timeout: 60000 })
    await toolRow.click()

    await expect(inspector.getByText('Sanitized Input')).toBeVisible()
    await expect(inspector.locator('pre').first()).toBeVisible()
  })

  test('5. Verify execution logs rendered line matches server logs endpoint', async ({ page }) => {
    await page.goto(`/workflows/runs/${runId}`)

    await page.getByRole('button', { name: /Execution Logs/i }).click()

    const logsRes = await page.request.get(`/api/backend/workflows/runs/${runId}/logs?limit=5`)
    if (!logsRes.ok()) {
      throw new Error(`Failed to fetch server logs: status ${logsRes.status()} ${await logsRes.text()}`)
    }
    const logsData = await logsRes.json()
    expect(logsData.items.length).toBeGreaterThan(0)

    const targetMessage = logsData.items[0].message
    await expect(page.locator(`text=${targetMessage}`).first()).toBeVisible({ timeout: 30000 })
  })

  test('6. Verify delegation view handles delegations or documented empty state', async ({ page }) => {
    await page.goto(`/workflows/runs/${runId}`)
    await page.getByRole('button', { name: 'Delegation Graph' }).click()

    const delRes = await page.request.get(`/api/backend/workflows/runs/${runId}/delegations`)
    if (!delRes.ok()) {
      throw new Error(`Failed to fetch delegations: status ${delRes.status()} ${await delRes.text()}`)
    }
    const delegations = await delRes.json()

    if (Array.isArray(delegations) && delegations.length > 0) {
      const edgeLabel = page.getByTestId('delegation-edge-label').first()
      await expect(edgeLabel).toBeVisible()
      await edgeLabel.click()
      await expect(page.locator('[data-testid="delegation-node"]').first()).toBeVisible()
    } else {
      await expect(page.locator('.vue-flow')).toBeVisible()
    }
  })

  test('7. Verify artifacts evidence provenance and click navigation to agent inspector', async ({ page }) => {
    await page.goto(`/workflows/runs/${runId}`)

    await page.getByRole('button', { name: /Artifacts/i }).click()
    const evidenceCards = page.locator('[data-testid="artifact-card"][data-artifact-type="evidence"]')
    await expect(evidenceCards.first()).toBeVisible({ timeout: 60000 })

    const sourceLink = evidenceCards.first().locator('a[target="_blank"]')
    await expect(sourceLink).toBeVisible()
    const href = await sourceLink.getAttribute('href')
    expect(href).not.toBeNull()
    expect(href!.length).toBeGreaterThan(0)

    const agentNavBtn = evidenceCards.first().locator('[data-testid="artifact-agent-link"]')
    await expect(agentNavBtn).toBeVisible()
    await agentNavBtn.click()

    await expect(page.locator('aside')).toBeVisible()
    await expect(page.getByText('Agent Execution Inspector')).toBeVisible()
  })

  test('8. Verify all workspace sections from contract have non-empty data', async ({ page }) => {
    await expect.poll(async () => {
      const res = await page.request.get(`/api/backend/workspaces/${workspaceId}`)
      if (!res.ok()) {
        throw new Error(`Workspace polling failed: status ${res.status()} ${await res.text()}`)
      }
      const data = await res.json()
      return data.status
    }, { timeout: 900000, intervals: [3000, 6000] }).toBe('ready')

    const roleRes = await page.request.get(`/api/backend/workspaces/${workspaceId}/role`)
    if (!roleRes.ok()) throw new Error(`Role failed: ${roleRes.status()}`)
    const roleData = await roleRes.json()
    expect(roleData.title.length).toBeGreaterThan(0)

    const matchRes = await page.request.get(`/api/backend/workspaces/${workspaceId}/match`)
    if (!matchRes.ok()) throw new Error(`Match failed: ${matchRes.status()}`)
    const matchData = await matchRes.json()
    expect(typeof matchData.overall_match_ratio).toBe('number')

    const compRes = await page.request.get(`/api/backend/workspaces/${workspaceId}/company`)
    if (!compRes.ok()) throw new Error(`Company failed: ${compRes.status()}`)
    const compData = await compRes.json()
    expect(compData.tech_stack?.length).toBeGreaterThanOrEqual(1)

    const questRes = await page.request.get(`/api/backend/workspaces/${workspaceId}/questions`)
    if (!questRes.ok()) throw new Error(`Questions failed: ${questRes.status()}`)
    const questData = await questRes.json()
    expect(questData.length).toBeGreaterThanOrEqual(1)

    const learnRes = await page.request.get(`/api/backend/workspaces/${workspaceId}/learning`)
    if (!learnRes.ok()) throw new Error(`Learning failed: ${learnRes.status()}`)
    const learnData = await learnRes.json()
    expect(learnData.length).toBeGreaterThanOrEqual(1)

    const srcRes = await page.request.get(`/api/backend/workspaces/${workspaceId}/sources`)
    if (!srcRes.ok()) throw new Error(`Sources failed: ${srcRes.status()}`)
    const srcData = await srcRes.json()
    expect(srcData.sources?.length).toBeGreaterThanOrEqual(1)
  })

  test('9. Dynamic run execution: plan.created, graph expansion, and rev marker', async ({ page }) => {
    test.setTimeout(2_100_000)
    await page.goto('/workflows')
    await page.getByRole('button', { name: 'New Workflow Run' }).click()
    const dialog = page.getByRole('dialog')
    await expect(dialog).toBeVisible()
    await dialog.getByRole('button', { name: /Dynamic Autonomous/ }).click()

    const select = dialog.locator('select')
    const wsOption = select.locator(`option[value="${workspaceId}"]`)
    await expect(wsOption).toHaveCount(1)
    await select.selectOption(workspaceId)

    await dialog.getByRole('button', { name: 'Launch Run' }).click()
    await page.waitForURL(/\/workflows\/runs\/(.+)/)

    const match = page.url().match(/\/workflows\/runs\/([^?#]+)/)
    expect(match).not.toBeNull()
    const dynamicRunId = match![1]

    let historyEvents: Array<Record<string, unknown>> = []
    await expect.poll(async () => {
      const res = await page.request.get(`/api/backend/workflows/runs/${dynamicRunId}/events/history?limit=100`)
      if (!res.ok()) {
        throw new Error(`History polling failed: status ${res.status()} ${await res.text()}`)
      }
      const data = await res.json()
      historyEvents = (data.events || []) as Array<Record<string, unknown>>
      return historyEvents.some((e) => e.type === 'plan.created')
    }, { timeout: 600000, intervals: [3000, 5000] }).toBe(true)
    const planCreatedEv = historyEvents.find((e) => e.type === 'plan.created')
    expect(planCreatedEv).toBeDefined()
    const payload = planCreatedEv?.payload as Record<string, unknown> | undefined
    const tasks = (payload?.tasks || []) as Array<Record<string, unknown>>
    const expectedTaskIds = new Set<string>()
    for (const t of tasks) {
      if (typeof t.id === 'string') expectedTaskIds.add(t.id)
      else if (typeof t.task_id === 'string') expectedTaskIds.add(t.task_id)
    }

    const taskAddedEvents = historyEvents.filter((e) => e.type === 'task.added')
    for (const ev of taskAddedEvents) {
      const p = ev.payload as Record<string, unknown> | undefined
      if (typeof p?.task_id === 'string') expectedTaskIds.add(p.task_id)
    }

    await expect.poll(async () => {
      const graphRes = await page.request.get(`/api/backend/workflows/runs/${dynamicRunId}/graph`)
      if (!graphRes.ok()) {
        throw new Error(`Graph polling failed: status ${graphRes.status()} ${await graphRes.text()}`)
      }
      const graphData = await graphRes.json()
      const graphNodes = (graphData.nodes || []) as Array<Record<string, unknown>>
      const graphNodeIds = new Set(graphNodes.map((n) => String(n.id)))

      for (const expectedId of expectedTaskIds) {
        if (!graphNodeIds.has(expectedId)) {
          return false
        }
      }
      return true
    }, { timeout: 30000, intervals: [1000, 2000] }).toBe(true)
    const tasksAddedCount = taskAddedEvents.length
    if (tasksAddedCount > 0) {
      await page.goto(`/workflows/runs/${dynamicRunId}`)
      const revBadge = page.locator('[data-testid="rev-badge"]').first()
      await expect(revBadge).toBeVisible({ timeout: 60000 })
      await expect(revBadge).toHaveText(/rev [2-9]/)
    }

    let dynamicRunData: Record<string, unknown> = {}
    await expect.poll(async () => {
      const res = await page.request.get(`/api/backend/workflows/runs/${dynamicRunId}`)
      if (!res.ok()) {
        throw new Error(`Dynamic run polling failed: status ${res.status()}`)
      }
      const data = (await res.json()) as Record<string, unknown>
      dynamicRunData = data
      return data.status
    }, { timeout: 1_800_000, intervals: [3000, 6000] }).toBe('completed')

    expect(dynamicRunData.degraded).toBe(false)
    if (dynamicRunData.degraded) {
      const reasons = Array.isArray(dynamicRunData.degraded_reasons) ? dynamicRunData.degraded_reasons.join(', ') : 'Unknown degradation'
      throw new Error(`Dynamic run completed in degraded state: ${reasons}`)
    }

    const dynamicGraphRes = await page.request.get(`/api/backend/workflows/runs/${dynamicRunId}/graph`)
    if (!dynamicGraphRes.ok()) {
      throw new Error(`Failed to fetch dynamic task graph: status ${dynamicGraphRes.status()}`)
    }
    const dynamicGraph = (await dynamicGraphRes.json()) as { nodes?: Array<Record<string, unknown>> }
    for (const node of dynamicGraph.nodes || []) {
      const nodeData = (node.data || {}) as Record<string, unknown>
      const isRequired = nodeData.required !== false
      const nodeStatus = nodeData.status
      if (isRequired) {
        expect(['completed', 'skipped']).toContain(nodeStatus)
      }
    }

    const dynAgentsRes = await page.request.get(`/api/backend/workflows/runs/${dynamicRunId}/agents`)
    if (!dynAgentsRes.ok()) {
      throw new Error(`Failed to fetch dynamic agents: status ${dynAgentsRes.status()}`)
    }
    const dynAgents = ((await dynAgentsRes.json()) || []) as Array<Record<string, unknown>>
    for (const a of dynAgents) {
      expect(a.status).not.toBe('running')
      expect(a.status).not.toBe('pending')
    }

    const dynDelsRes = await page.request.get(`/api/backend/workflows/runs/${dynamicRunId}/delegations`)
    if (!dynDelsRes.ok()) {
      throw new Error(`Failed to fetch dynamic delegations: status ${dynDelsRes.status()}`)
    }
    const dynDels = ((await dynDelsRes.json()) || []) as Array<Record<string, unknown>>
    for (const d of dynDels) {
      expect(['completed', 'failed', 'cancelled', 'rejected']).toContain(d.status)
    }
  })
})
