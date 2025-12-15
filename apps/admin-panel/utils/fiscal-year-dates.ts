type DateInput = string | null | undefined

const normalizeDateInputToUTC = (value: string) =>
  value.includes("T") ? value : `${value}T00:00:00Z`

export function parseUTCDate(value: DateInput): Date | null {
  if (!value) return null
  const normalized = normalizeDateInputToUTC(value.trim())
  const date = new Date(normalized)
  return Number.isNaN(date.getTime()) ? null : date
}

export function getUTCYear(value: DateInput): number | null {
  const date = parseUTCDate(value)
  return date ? date.getUTCFullYear() : null
}

export function formatUTCMonthName(value: DateInput, locale: string): string | null {
  const date = parseUTCDate(value)
  if (!date) return null
  return date.toLocaleString(locale, { month: "long", timeZone: "UTC" })
}

export function formatUTCMonthYear(value: DateInput, locale: string): string | null {
  const date = parseUTCDate(value)
  if (!date) return null
  return date.toLocaleString(locale, {
    month: "long",
    year: "numeric",
    timeZone: "UTC",
  })
}
