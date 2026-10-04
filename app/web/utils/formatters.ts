/**
 * Root utility functions shared across features.
 */

export const formatDate = (dateString?: string | null): string => {
  if (!dateString) return 'N/A'
  try {
    return new Date(dateString).toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    })
  } catch {
    return dateString
  }
}

export const formatDateTime = (dateString?: string | null): string => {
  if (!dateString) return 'N/A'
  try {
    return new Date(dateString).toLocaleString()
  } catch {
    return dateString
  }
}

export const formatPercentage = (ratio?: number | null): number => {
  if (ratio == null || isNaN(ratio)) return 0
  return Math.round(ratio * 100)
}

