import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export const formatDate = (
  dateInput: string | number | Date,
  options: { includeTime: boolean } = { includeTime: true },
): string => {
  const date = dateInput instanceof Date ? dateInput : new Date(dateInput)

  if (Number.isNaN(date.getTime())) return "Invalid date"
  const locale =
    typeof document !== "undefined"
      ? document.documentElement.lang || navigator.language || "en-US"
      : "en-US"

  const base: Intl.DateTimeFormatOptions = {
    dateStyle: "medium",
  }
  const opts: Intl.DateTimeFormatOptions = options.includeTime
    ? { ...base, timeStyle: "short" }
    : base

  return new Intl.DateTimeFormat(locale, opts).format(date)
}
