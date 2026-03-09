"use client"

import React from "react"

const HIDDEN_KEYS = ["type"]

const flattenPayload = (
  obj: Record<string, unknown>,
  prefix = "",
): [string, string][] => {
  const entries: [string, string][] = []
  for (const [key, value] of Object.entries(obj)) {
    if (!prefix && HIDDEN_KEYS.includes(key)) continue
    const fullKey = prefix ? `${prefix}.${key}` : key
    if (value !== null && typeof value === "object" && !Array.isArray(value)) {
      entries.push(...flattenPayload(value as Record<string, unknown>, fullKey))
    } else {
      entries.push([fullKey, String(value)])
    }
  }
  return entries
}

export const EventPayload: React.FC<{ payload: Record<string, unknown> }> = ({
  payload,
}) => {
  const entries = flattenPayload(payload)

  if (entries.length === 0) return null

  return (
    <div className="text-muted-foreground text-sm space-y-0.5">
      {entries.map(([key, value]) => (
        <div key={key}>
          {key}: {value}
        </div>
      ))}
    </div>
  )
}

export const renderEventPayload = (
  payload: Record<string, unknown>,
): React.ReactNode | null => {
  if (flattenPayload(payload).length === 0) return null
  return <EventPayload payload={payload} />
}
