"use client"

import React, { useState } from "react"
import { useTranslations } from "next-intl"

const HIDDEN_KEYS = ["type"]

const isIdKey = (key: string): boolean => key === "id" || key.endsWith("_id") || key.endsWith("_ids")

export const EventPayload: React.FC<{ payload: Record<string, unknown> }> = ({
  payload,
}) => {
  const t = useTranslations("Common")
  const [expanded, setExpanded] = useState(false)

  const entries = Object.entries(payload).filter(
    ([key]) => !HIDDEN_KEYS.includes(key),
  )
  const mainEntries = entries.filter(([key]) => !isIdKey(key))
  const idEntries = entries.filter(([key]) => isIdKey(key))

  if (entries.length === 0) return null

  return (
    <div className="text-muted-foreground text-sm space-y-0.5">
      {mainEntries.map(([key, value]) => (
        <div key={key}>
          {key}: {String(value)}
        </div>
      ))}
      {idEntries.length > 0 && (
        <>
          {expanded &&
            idEntries.map(([key, value]) => (
              <div key={key}>
                {key}: {String(value)}
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
