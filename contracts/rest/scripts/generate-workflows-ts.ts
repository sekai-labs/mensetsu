import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { compile } from 'json-schema-to-typescript'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

async function generateContracts() {
  const schemaPath = path.resolve(__dirname, '../schema/workflows.v1.json')
  const outputPath = path.resolve(__dirname, '../src/generated/workflows.v1.ts')

  if (!fs.existsSync(schemaPath)) {
    throw new Error(`Schema file not found at ${schemaPath}`)
  }

  const rawSchema = JSON.parse(fs.readFileSync(schemaPath, 'utf8'))
  const defs = rawSchema.$defs || {}

  const properties: Record<string, { $ref: string }> = {}
  for (const defName of Object.keys(defs)) {
    properties[defName] = { $ref: `#/$defs/${defName}` }
  }
  const rootSchema = {
    ...rawSchema,
    properties,
  }
  let compiled = await compile(rootSchema, 'WorkflowContractRoot', {
    bannerComment: '',
    unreachableDefinitions: true,
    style: {
      singleQuote: true,
      semi: false,
    },
  })
  compiled = compiled.replace(/\/\*\*[\s\S]*?\*\//g, '').replace(/^\s*[\r\n]/gm, '\n')
  const eventDefNames = Object.keys(defs).filter((k) => k.endsWith('Event'))

  const eventTypeLiterals = eventDefNames
    .map((name) => {
      const typeProp = defs[name]?.properties?.type
      const val = typeProp?.const || (typeProp?.enum && typeProp?.enum[0])
      return val ? `  | '${val}'` : null
    })
    .filter(Boolean)
    .join('\n')

  const eventUnionMembers = eventDefNames.map((name) => `  | ${name}`).join('\n')

  const extraExports = `
export type WorkflowType = 'static' | 'dynamic' | 'hybrid'
export type RunStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
export type TaskStatus =
  | 'pending'
  | 'queued'
  | 'running'
  | 'completed'
  | 'failed'
  | 'retrying'
  | 'blocked'
  | 'cancelled'
  | 'skipped'
export type ExecutionStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
export type ToolStatus = 'pending' | 'running' | 'completed' | 'failed'
export type DelegationStatus = 'requested' | 'accepted' | 'running' | 'completed' | 'failed' | 'rejected'
export type LogSeverity = 'debug' | 'info' | 'warn' | 'error'
export type ArtifactType =
  | 'research_report'
  | 'finding'
  | 'evidence'
  | 'question_set'
  | 'recommendation'
  | 'validation_result'

export type WorkflowEventType =
${eventTypeLiterals}

export const WORKFLOW_EVENT_TYPES: readonly WorkflowEventType[] = [
${eventDefNames
  .map((name) => {
    const typeProp = defs[name]?.properties?.type
    const val = typeProp?.const || (typeProp?.enum && typeProp?.enum[0])
    return val ? `  '${val}',` : null
  })
  .filter(Boolean)
  .join('\n')}
] as const

export type WorkflowEvent =
${eventUnionMembers}
`

  const finalOutput = compiled + '\n' + extraExports
  fs.mkdirSync(path.dirname(outputPath), { recursive: true })
  fs.writeFileSync(outputPath, finalOutput, 'utf8')
  console.log(`Generated ${outputPath} from ${schemaPath}`)
}

generateContracts().catch((err) => {
  console.error('Failed to generate contracts:', err)
  process.exit(1)
})
