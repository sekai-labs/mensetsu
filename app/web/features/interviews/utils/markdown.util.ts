import { marked } from 'marked'

marked.setOptions({
  gfm: true,
  breaks: true,
})

export function renderMarkdown(content: string | null | undefined): string {
  if (!content || typeof content !== 'string') {
    return ''
  }

  const sanitized = content
    .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')
    .replace(/javascript:/gi, '')
    .replace(/onload=/gi, '')
    .replace(/onerror=/gi, '')

  try {
    const rawHtml = marked.parse(sanitized) as string
    return rawHtml
  } catch (err) {
    console.warn('Failed to parse markdown:', err)
    return sanitized.replace(/</g, '&lt;').replace(/>/g, '&gt;')
  }
}
