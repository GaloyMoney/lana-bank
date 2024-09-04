import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

import { InterestInterval, Period } from "./graphql/generated"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export const currencyConverter = {
  centsToUsd: (cents: number) => {
    return Number((cents / 100).toFixed(2))
  },

  btcToSatoshi: (btc: number) => {
    return Number((btc * 100000000).toFixed(0))
  },

  satoshiToBtc: (satoshi: number) => {
    return satoshi / 100000000
  },

  usdToCents: (usd: number) => {
    return Number((usd * 100).toFixed(0))
  },
}

export function formatCurrency({
  amount,
  currency,
}: {
  amount: number
  currency: string
}) {
  if (currency === "SATS" && amount >= 100_000_000)
    return formatCurrency({ amount: amount / 100_000_000, currency: "BTC" })

  if (currency === "SATS") {
    return (
      new Intl.NumberFormat("en-US", {
        maximumFractionDigits: 0,
        useGrouping: true,
      }).format(amount) + " SATS"
    )
  }

  if (currency === "BTC") {
    return `${amount.toFixed(8)} BTC`
  }

  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency,
  }).format(amount)
}

export const formatDate = (isoDateString: string): string => {
  if (isoDateString === "-") return "-"

  const date = new Date(isoDateString)
  const options: Intl.DateTimeFormatOptions = {
    year: "numeric",
    month: "long",
    day: "numeric",
  }

  const formattedDate = date.toLocaleDateString("en-US", options)
  const formattedTime = date
    .toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      hour12: true,
    })
    .toUpperCase()

  return `${formattedDate}, ${formattedTime}`
}

export const isUUID = (str: string) => {
  const uuidRegex =
    /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i
  return uuidRegex.test(str)
}

export const isEmail = (str: string) => {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  return emailRegex.test(str)
}

export const formatRole = (role: string) => {
  return role
    .toLowerCase()
    .split("_")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ")
}

export const formatPeriod = (period: Period) => {
  return period.charAt(0).toUpperCase() + period.slice(1).toLowerCase()
}

export const formatInterval = (interval: InterestInterval) => {
  return interval
    .toLowerCase()
    .split("_")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ")
}
