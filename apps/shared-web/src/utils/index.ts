import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

const getLocale = () =>
  typeof document !== "undefined"
    ? document.documentElement.lang || navigator.language || "en-US"
    : "en-US";

// Shared pattern for date-only strings (YYYY-MM-DD)
const ISO_DATE_PATTERN = /^\d{4}-\d{2}-\d{2}$/;

const isDateOnlyString = (input: unknown): input is string =>
  typeof input === "string" && ISO_DATE_PATTERN.test(input);

export const formatDate = (
  dateInput: string | number | Date,
  options: { includeTime: boolean } = { includeTime: true }
): string => {
  // Date-only strings (e.g., "2028-06-17") should be formatted in UTC
  // to avoid timezone shifts that could change the displayed date
  if (isDateOnlyString(dateInput)) {
    return formatUTCDateOnly(dateInput) ?? "Invalid date";
  }

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

export const formatSpacedSentenceCaseFromSnakeCase = (str: string): string => {
  return str
    .replace(/_/g, " ") // Replace underscores with spaces
    .replace(/(?<!\S)\p{L}/gu, (char) => char.toUpperCase()); // Capitalize the first letter of each word
};

// Date-only helpers (UTC-safe)
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
