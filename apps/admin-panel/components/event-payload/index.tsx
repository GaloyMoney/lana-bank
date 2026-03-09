"use client"

import React, { useState } from "react"
import { useTranslations } from "next-intl"

const HIDDEN_KEYS = ["type"]

const isIdKey = (key: string): boolean =>
  key === "id" || key.endsWith("_id") || key.endsWith("_ids")

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
  const t = useTranslations("Common")
  const [expanded, setExpanded] = useState(false)

  const entries = flattenPayload(payload)
  const mainEntries = entries.filter(([key]) => !isIdKey(key))
  const idEntries = entries.filter(([key]) => isIdKey(key))

  if (entries.length === 0) return null

  return (
    <div className="text-muted-foreground text-sm space-y-0.5">
      {mainEntries.map(([key, value]) => (
        <div key={key}>
          {key}: {value}
        </div>
      ))}
      {idEntries.length > 0 && (
        <>
          {expanded &&
            idEntries.map(([key, value]) => (
              <div key={key}>
                {key}: {value}
              </div>
            ))}
          <button
            type="button"
            className="text-xs text-primary hover:underline"
            onClick={(e) => {
              e.stopPropagation()
              setExpanded(!expanded)
            }}
          >
            {expanded ? t("hideIds") : t("showIds")}
          </button>
        </>
      )}
    </div>
  )
}
