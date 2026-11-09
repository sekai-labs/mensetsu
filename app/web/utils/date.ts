const DISPLAY_LOCALE = 'en-US'
const DISPLAY_TIME_ZONE = 'UTC'

const dateFormatter = new Intl.DateTimeFormat(DISPLAY_LOCALE, {
  year: 'numeric',
  month: 'numeric',
  day: 'numeric',
  timeZone: DISPLAY_TIME_ZONE,
})

const dateTimeFormatter = new Intl.DateTimeFormat(DISPLAY_LOCALE, {
  dateStyle: 'medium',
  timeStyle: 'short',
  timeZone: DISPLAY_TIME_ZONE,
})

const toDisplayDate = (value: string | Date): Date | null => {
  const date = value instanceof Date ? value : new Date(value)
  return Number.isNaN(date.getTime()) ? null : date
}

export const formatDisplayDate = (value: string | Date): string => {
  const date = toDisplayDate(value)
  return date ? `${dateFormatter.format(date)} UTC` : ''
}

export const formatDisplayDateTime = (value: string | Date): string => {
  const date = toDisplayDate(value)
  return date ? `${dateTimeFormatter.format(date)} UTC` : ''
}
