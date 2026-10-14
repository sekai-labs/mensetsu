import { describe, it, expect } from 'vitest'
import { renderMarkdown } from '../features/interviews/utils/markdown.util'
import {
  getSkillNodeLabel,
  isSkillNodeMatched,
  getSkillNodeStatusCategory,
} from '../features/interviews/utils/interview.util'
import type { KnowledgeSkillNode, KnowledgeEdge } from '../features/interviews/models/interview.model'

describe('Markdown Renderer Utility', () => {
  it('returns empty string on null or undefined input', () => {
    expect(renderMarkdown(null)).toBe('')
    expect(renderMarkdown(undefined)).toBe('')
    expect(renderMarkdown('')).toBe('')
  })

  it('renders bold, headers, code, lists, and tables correctly', () => {
    const md = `
# Architecture Deep Dive

This is an **in-depth** review of consensus protocols.

## Key Principles
- Single-leader replication
- Quorum reads & writes

\`\`\`rust
fn heartbeat() -> bool {
    true
}
\`\`\`
`
    const html = renderMarkdown(md)
    expect(html).toContain('<h1')
    expect(html).toContain('Architecture Deep Dive')
    expect(html).toContain('<strong>in-depth</strong>')
    expect(html).toContain('Key Principles')
    expect(html).toContain('<ul')
    expect(html).toContain('Single-leader replication')
    expect(html).toContain('<code')
    expect(html).toContain('heartbeat')
  })

  it('sanitizes script tags and dangerous event handlers to prevent XSS', () => {
    const malicious = `
# Dangerous Injection
<script>alert('xss')</script>
<img src="x" onerror="alert(1)" />
[Link](javascript:alert(1))
`
    const html = renderMarkdown(malicious)
    expect(html).not.toContain('<script>')
    expect(html).not.toContain('alert(\'xss\')</script>')
    expect(html).not.toContain('javascript:')
    expect(html).not.toContain('onerror=')
  })
})

describe('Knowledge & Skill Helpers', () => {
  it('extracts skill node label accurately across label, name, slug, and id', () => {
    const nodeWithName: KnowledgeSkillNode = {
      id: 'skill_1',
      name: 'Rust Tokio Concurrency',
      slug: 'rust-tokio',
      category: 'systems',
      importance: 'mandatory',
    }
    expect(getSkillNodeLabel(nodeWithName)).toBe('Rust Tokio Concurrency')

    const nodeWithSlugOnly: KnowledgeSkillNode = {
      id: 'skill_2',
      slug: 'surrealdb-graph',
      category: 'database',
      importance: 'mandatory',
    }
    expect(getSkillNodeLabel(nodeWithSlugOnly)).toBe('surrealdb-graph')

    const nodeWithLabel: KnowledgeSkillNode = {
      id: 'skill_3',
      label: 'Temporal Workflows',
      category: 'orchestration',
      importance: 'mandatory',
    }
    expect(getSkillNodeLabel(nodeWithLabel)).toBe('Temporal Workflows')
  })

  it('identifies matched, in-progress, and gap skill statuses properly', () => {
    const verifiedNode: KnowledgeSkillNode = {
      id: '1',
      name: 'Go',
      category: 'backend',
      importance: 'mandatory',
      candidate_status: 'verified',
    }
    expect(isSkillNodeMatched(verifiedNode)).toBe(true)
    expect(getSkillNodeStatusCategory(verifiedNode)).toBe('matched')

    const inProgressNode: KnowledgeSkillNode = {
      id: '2',
      name: 'Distributed Tracing',
      category: 'observability',
      importance: 'preferred',
      candidate_status: 'in_progress',
    }
    expect(isSkillNodeMatched(inProgressNode)).toBe(false)
    expect(getSkillNodeStatusCategory(inProgressNode)).toBe('in_progress')

    const gapNode: KnowledgeSkillNode = {
      id: '3',
      name: 'eBPF',
      category: 'kernel',
      importance: 'optional',
      candidate_status: 'gap',
    }
    expect(isSkillNodeMatched(gapNode)).toBe(false)
    expect(getSkillNodeStatusCategory(gapNode)).toBe('gap')

    const booleanMatchedNode: KnowledgeSkillNode = {
      id: '4',
      name: 'PostgreSQL',
      category: 'database',
      importance: 'mandatory',
      is_matched: true,
    }
    expect(isSkillNodeMatched(booleanMatchedNode)).toBe(true)
    expect(getSkillNodeStatusCategory(booleanMatchedNode)).toBe('matched')
  })
})
