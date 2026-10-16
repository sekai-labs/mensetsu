import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import Ajv from 'ajv'
import {
  parseWorkflowEvent,
  parseSseStream,
  parseNdjsonEvents,
  parseWorkflowRunResponse,
  parseTaskGraphResponse,
  parseDelegationGraphResponse,
} from '../features/workflows/api/contract'

const schemaPath = path.resolve(__dirname, '../../../contracts/rest/schema/workflows.v1.json')
const rawSchema = JSON.parse(fs.readFileSync(schemaPath, 'utf8'))
const ajv = new Ajv({ strict: false, allErrors: true })
ajv.addSchema(rawSchema, 'workflows.v1.json')

const fixturesDir = path.resolve(__dirname, '../../../contracts/rest/fixtures/workflows.v1')

function validateAgainstDef(defName: string, data: unknown) {
  const validate = ajv.getSchema(`workflows.v1.json#/$defs/${defName}`)
  if (!validate) {
    throw new Error(`Schema definition ${defName} not found in workflows.v1.json`)
  }
  const valid = validate(data)
  expect(valid).toBe(true)
}

describe('Workflow Contract Adapter & Real Fixtures', () => {
  const runFixture = JSON.parse(fs.readFileSync(path.join(fixturesDir, 'run.json'), 'utf8'))
  const graphFixture = JSON.parse(fs.readFileSync(path.join(fixturesDir, 'graph.json'), 'utf8'))
  const delegationGraphFixture = JSON.parse(fs.readFileSync(path.join(fixturesDir, 'delegations-graph.json'), 'utf8'))
  const eventsNdjson = fs.readFileSync(path.join(fixturesDir, 'events.ndjson'), 'utf8')
  const sseText = fs.readFileSync(path.join(fixturesDir, 'sse.txt'), 'utf8')

  it('validates real fixtures from intelligence worker against workflows.v1.json', () => {
    validateAgainstDef('WorkflowRun', runFixture)
    validateAgainstDef('TaskGraphResponse', graphFixture)
    validateAgainstDef('DelegationGraphResponse', delegationGraphFixture)
  })

  it('parses real run snapshot fixture', () => {
    const run = parseWorkflowRunResponse(runFixture)
    expect(run.workflow_id).toBe('interview-preparation-static')
    expect(run.status).toBe('completed')
    expect(run.last_seq).toBe(runFixture.last_seq)
  })

  it('parses real task graph and delegation graph fixtures', () => {
    const taskGraph = parseTaskGraphResponse(graphFixture)
    expect(Array.isArray(taskGraph.nodes)).toBe(true)

    const delegationGraph = parseDelegationGraphResponse(delegationGraphFixture)
    expect(Array.isArray(delegationGraph.nodes)).toBe(true)
  })

  it('parses real events.ndjson fixture stream into valid events', () => {
    const events = parseNdjsonEvents(eventsNdjson)
    expect(events.length).toBeGreaterThan(0)
    expect(events[0].type).toBe('workflow.created')
    expect(events[0].seq).toBe(1)
  })

  it('parses real sse.txt stream including monotonic id and event types', () => {
    const frames = parseSseStream(sseText)
    expect(frames.length).toBeGreaterThan(0)
    expect(frames[0].id).toBe(1)
    expect(frames[0].event).toBe('workflow.created')
    expect(frames[0].rawEvent?.type).toBe('workflow.created')
  })

  it('rejects unknown event types with logged warning without crash', () => {
    const unknownEvent = {
      ...eventsNdjson.split('\n')[0] ? JSON.parse(eventsNdjson.split('\n')[0]) : {},
      type: 'alien.invaded',
    }
    const result = parseWorkflowEvent(unknownEvent)
    expect(result).toBeNull()
  })
})
