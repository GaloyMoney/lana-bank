import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

const getLocale = () =>
  typeof document !== "undefined"
    ? document.documentElement.lang || navigator.language || "en-US"
    : "en-US";

export const formatDate = (
  dateInput: string | number | Date,
  options: { includeTime: boolean } = { includeTime: true }
): string => {
  const date = dateInput instanceof Date ? dateInput : new Date(dateInput);

  if (Number.isNaN(date.getTime())) return "Invalid date";
  const locale = getLocale();

  const base: Intl.DateTimeFormatOptions = {
    dateStyle: "medium",
  };
  const opts: Intl.DateTimeFormatOptions = options.includeTime
    ? { ...base, timeStyle: "short" }
    : base;

  return new Intl.DateTimeFormat(locale, opts).format(date);
};

// Formatter: always 4-digit year, 2-digit month, 2-digit day  → 2025-12-07
const isoLike = new Intl.DateTimeFormat("en-CA", {
  year: "numeric",
  month: "2-digit",
  day: "2-digit",
});

/** Convert a Date → "YYYY-MM-DD" (UTC, no time part). */
export const toISODateString = (date: Date): string => isoLike.format(date);

/**
 * Convert "YYYY-MM-DD" → Date at 00:00:00 UTC.
 * Throws if the string doesn’t match the exact pattern or
 * the calendar values are out of range.
 */
export const fromISODateString = (dateString: string): Date => {
  // Strictly require 4-2-2 digits with hyphens.
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(dateString);
  if (!match) {
    throw new Error(`Invalid ISO-like date: ${dateString}`);
  }

  const [, yearStr, monthStr, dayStr] = match;
  const year = Number(yearStr);
  const month = Number(monthStr);
  const day = Number(dayStr);

  if (month < 1 || month > 12 || day < 1 || day > 31) {
    throw new Error(`Invalid calendar date: ${dateString}`);
  }

  // Months are 0-based in JavaScript Dates.
  return new Date(Date.UTC(year, month - 1, day));
};

export const formatSpacedSentenceCaseFromSnakeCase = (str: string): string => {
  return str
    .replace(/_/g, " ") // Replace underscores with spaces
    .replace(/(?<!\S)\p{L}/gu, (char) => char.toUpperCase()); // Capitalize the first letter of each word
};

// Date-only helpers (UTC-safe).
type DateInput = string | null | undefined;

const normalizeDateInputToUTC = (value: string) =>
  value.includes("T") ? value : `${value}T00:00:00Z`;

export function parseUTCDate(value: DateInput): Date | null {
  if (!value) return null;
  const normalized = normalizeDateInputToUTC(value.trim());
  const date = new Date(normalized);
  return Number.isNaN(date.getTime()) ? null : date;
}

export function getUTCYear(value: DateInput): number | null {
  const date = parseUTCDate(value);
  return date ? date.getUTCFullYear() : null;
}

export function formatUTCMonthName(
  value: DateInput,
  locale: string
): string | null {
  const date = parseUTCDate(value);
  if (!date) return null;
  return date.toLocaleString(locale, { month: "long", timeZone: "UTC" });
}

export function formatUTCMonthYear(
  value: DateInput,
  locale: string
): string | null {
  const date = parseUTCDate(value);
  if (!date) return null;
  return date.toLocaleString(locale, {
    month: "long",
    year: "numeric",
    timeZone: "UTC",
  });
}

export function formatUTCDateOnly(value: DateInput): string | null {
  const date = parseUTCDate(value);
  if (!date) return null;
  return date.toLocaleDateString(getLocale(), {
    timeZone: "UTC",
    dateStyle: "medium",
  });
}
