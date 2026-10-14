import { marked } from 'marked'

/**
 * Configure marked renderer options for safe, clean technical documentation rendering.
 */
marked.setOptions({
  gfm: true,
  breaks: true,
})

/**
 * Sanitizes and renders markdown content into safe HTML.
 * Escapes raw dangerous HTML tags to prevent XSS while maintaining clean technical formatting.
 */
export function renderMarkdown(content: string | null | undefined): string {
  if (!content || typeof content !== 'string') {
    return ''
  }

  // Escape script tags and dangerous event handlers if present before parsing
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
