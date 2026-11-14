import DOMPurify from 'isomorphic-dompurify'
import { marked } from 'marked'

marked.setOptions({
  gfm: true,
  breaks: true,
})

const escapeHtml = (value: string): string =>
  value.replace(/[&<>"']/g, (char) => {
    switch (char) {
      case '&':
        return '&amp;'
      case '<':
        return '&lt;'
      case '>':
        return '&gt;'
      case '"':
        return '&quot;'
      case "'":
        return '&#39;'
      default:
        return char
    }
  })

export function renderMarkdown(content: string | null | undefined): string {
  if (!content || typeof content !== 'string') {
    return ''
  }

  try {
    const rawHtml = marked.parse(content) as string
    return DOMPurify.sanitize(rawHtml, { USE_PROFILES: { html: true } })
  } catch (err) {
    console.warn('Failed to parse markdown:', err)
    return escapeHtml(content)
  }
}
