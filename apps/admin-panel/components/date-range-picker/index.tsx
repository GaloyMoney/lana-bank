"use client"

import { useCallback, useMemo, useState } from "react"
import { useTranslations } from "next-intl"
import { ChevronDownIcon } from "lucide-react"

import { Button } from "@lana/web/ui/button"
import { Calendar } from "@lana/web/ui/calendar"
import { Popover, PopoverContent, PopoverTrigger } from "@lana/web/ui/popover"

import { formatDate } from "@lana/web/utils"

export type DateRange = {
  from: string
  until: string
}

type DateRangeSelectorProps = {
  initialDateRange: DateRange
  onDateChange: (dateRange: DateRange) => void
}

const toDateString = (date: Date): string => {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, "0")
  const day = String(date.getDate()).padStart(2, "0")
  return `${year}-${month}-${day}`
}

export const parseDateString = (dateStr: string): Date => {
  const [year, month, day] = dateStr.split("-").map(Number)
  return new Date(year, month - 1, day)
}

type PresetKey =
  | "thisMonth"
  | "thisQuarter"
  | "ytd"
  | "lastMonth"
  | "lastQuarter"
  | "lastYear"

type Preset = {
  key: PresetKey
  getRange: () => DateRange
}

const QUARTER_NAMES = ["Q1", "Q2", "Q3", "Q4"]

const buildPresets = (): Preset[] => {
  const today = new Date()
  const year = today.getFullYear()
  const month = today.getMonth()
  const currentQuarter = Math.floor(month / 3)

  const presets: Preset[] = [
    {
      key: "thisMonth",
      getRange: () => ({
        from: toDateString(new Date(year, month, 1)),
        until: toDateString(today),
      }),
    },
    {
      key: "thisQuarter",
      getRange: () => ({
        from: toDateString(new Date(year, currentQuarter * 3, 1)),
        until: toDateString(today),
      }),
    },
    {
      key: "ytd",
      getRange: () => ({
        from: toDateString(new Date(year, 0, 1)),
        until: toDateString(today),
      }),
    },
  ]

  if (month > 0) {
    presets.push({
      key: "lastMonth",
      getRange: () => ({
        from: toDateString(new Date(year, month - 1, 1)),
        until: toDateString(new Date(year, month, 0)),
      }),
    })
  }

  if (currentQuarter > 0) {
    presets.push({
      key: "lastQuarter",
      getRange: () => ({
        from: toDateString(new Date(year, (currentQuarter - 1) * 3, 1)),
        until: toDateString(new Date(year, currentQuarter * 3, 0)),
      }),
    })
  }

  presets.push({
    key: "lastYear",
    getRange: () => ({
      from: toDateString(new Date(year - 1, 0, 1)),
      until: toDateString(new Date(year - 1, 11, 31)),
    }),
  })

  return presets
}

const formatPresetLabel = (
  t: ReturnType<typeof useTranslations<"DateRangePicker">>,
  key: PresetKey,
): string => {
  const now = new Date()
  const year = now.getFullYear()
  const month = now.getMonth()
  const currentQuarter = Math.floor(month / 3)
  const monthName = now.toLocaleString("default", { month: "long" })

  switch (key) {
    case "thisMonth":
      return t("preset.thisMonth", { month: monthName })
    case "thisQuarter":
      return t("preset.thisQuarter", { quarter: QUARTER_NAMES[currentQuarter] })
    case "ytd":
      return t("preset.ytd", { year: String(year) })
    case "lastMonth": {
      const prev = new Date(year, month - 1, 1)
      return t("preset.lastMonth", {
        month: prev.toLocaleString("default", { month: "long" }),
      })
    }
    case "lastQuarter":
      return t("preset.lastQuarter", {
        quarter: QUARTER_NAMES[currentQuarter - 1],
      })
    case "lastYear":
      return t("preset.lastYear", { year: String(year - 1) })
  }
}

type Selection =
  | { type: "preset"; key: PresetKey }
  | { type: "custom" }

const MATCHING_PRIORITY: PresetKey[] = [
  "ytd",
  "lastYear",
  "thisQuarter",
  "lastQuarter",
  "thisMonth",
  "lastMonth",
]

const findMatchingPreset = (
  range: DateRange,
  presets: Preset[],
): PresetKey | null => {
  for (const key of MATCHING_PRIORITY) {
    const preset = presets.find((p) => p.key === key)
    if (preset) {
      const presetRange = preset.getRange()
      if (presetRange.from === range.from && presetRange.until === range.until) {
        return preset.key
      }
    }
  }
  return null
}

export const getInitialDateRange = (): DateRange => {
  const today = new Date()
  const oneYearAgo = new Date(today.getFullYear() - 1, today.getMonth(), today.getDate())
  return {
    from: toDateString(oneYearAgo),
    until: toDateString(today),
  }
}

export const getYtdDateRange = (): DateRange => {
  const today = new Date()
  const startOfYear = new Date(today.getFullYear(), 0, 1)
  return {
    from: toDateString(startOfYear),
    until: toDateString(today),
  }
}

export const DateRangeSelector = ({
  initialDateRange,
  onDateChange,
}: DateRangeSelectorProps) => {
  const t = useTranslations("DateRangePicker")
  const [isOpen, setIsOpen] = useState(false)
  const [showCustom, setShowCustom] = useState(false)

  const presets = useMemo(() => buildPresets(), [])

  const [selection, setSelection] = useState<Selection>(() => {
    const match = findMatchingPreset(initialDateRange, presets)
    return match ? { type: "preset", key: match } : { type: "custom" }
  })

  const [dateRange, setDateRange] = useState<DateRange>(initialDateRange)

  const [selectedFrom, setSelectedFrom] = useState<Date | undefined>(
    parseDateString(initialDateRange.from),
  )
  const [selectedTo, setSelectedTo] = useState<Date | undefined>(
    parseDateString(initialDateRange.until),
  )

  const today = useMemo(() => {
    const date = new Date()
    date.setHours(0, 0, 0, 0)
    return date
  }, [])

  const handlePresetClick = useCallback(
    (preset: Preset) => {
      const range = preset.getRange()
      setSelection({ type: "preset", key: preset.key })
      setDateRange(range)
      setSelectedFrom(parseDateString(range.from))
      setSelectedTo(parseDateString(range.until))
      onDateChange(range)
      setShowCustom(false)
      setIsOpen(false)
    },
    [onDateChange],
  )

  const handleCustomApply = useCallback(() => {
    if (selectedFrom && selectedTo) {
      const range = {
        from: toDateString(selectedFrom),
        until: toDateString(selectedTo),
      }
      setSelection({ type: "custom" })
      setDateRange(range)
      onDateChange(range)
      setShowCustom(false)
      setIsOpen(false)
    }
  }, [selectedFrom, selectedTo, onDateChange])

  const handleOpenChange = useCallback((open: boolean) => {
    setIsOpen(open)
    if (!open) {
      setShowCustom(false)
    }
  }, [])

  const triggerLabel =
    selection.type === "preset"
      ? formatPresetLabel(t, selection.key)
      : `${formatDate(parseDateString(dateRange.from), { includeTime: false })} - ${formatDate(parseDateString(dateRange.until), { includeTime: false })}`

  return (
    <Popover open={isOpen} onOpenChange={handleOpenChange}>
      <PopoverTrigger asChild>
        <button className="flex items-center gap-2 rounded-md p-2 px-3 text-sm border cursor-pointer bg-muted hover:bg-muted/80">
          <span>{triggerLabel}</span>
          <ChevronDownIcon className="size-4 opacity-50" />
        </button>
      </PopoverTrigger>
      <PopoverContent className="w-auto p-0" align="start">
        {showCustom ? (
          <div className="flex flex-col">
            <div className="flex">
              <div className="border-r">
                <div className="p-3 text-sm font-medium">{t("fromDate")}</div>
                <Calendar
                  mode="single"
                  selected={selectedFrom}
                  onSelect={setSelectedFrom}
                  defaultMonth={selectedFrom}
                  disabled={(date) => date > today}
                  captionLayout="dropdown"
                />
              </div>
              <div>
                <div className="p-3 text-sm font-medium">{t("toDate")}</div>
                <Calendar
                  mode="single"
                  selected={selectedTo}
                  onSelect={setSelectedTo}
                  defaultMonth={selectedTo}
                  disabled={(date) =>
                    date > today || (selectedFrom ? date < selectedFrom : false)
                  }
                  captionLayout="dropdown"
                />
              </div>
            </div>
            <div className="border-t p-2 flex justify-between">
              <Button variant="ghost" size="sm" onClick={() => setShowCustom(false)}>
                {t("back")}
              </Button>
              <Button
                onClick={handleCustomApply}
                variant="ghost"
                size="sm"
                disabled={!selectedFrom || !selectedTo}
              >
                {t("apply")}
              </Button>
            </div>
          </div>
        ) : (
          <div className="flex flex-col py-1 min-w-[180px]">
            <div className="px-3 py-1.5 text-xs text-muted-foreground">
              {t("group.current")}
            </div>
            {presets
              .filter((p) =>
                (["thisMonth", "thisQuarter", "ytd"] as PresetKey[]).includes(p.key),
              )
              .map((preset) => (
                <button
                  key={preset.key}
                  className={`text-left px-3 py-1.5 text-sm hover:bg-accent cursor-pointer rounded-sm mx-1 ${
                    selection.type === "preset" && selection.key === preset.key
                      ? "bg-accent font-medium"
                      : ""
                  }`}
                  onClick={() => handlePresetClick(preset)}
                >
                  {formatPresetLabel(t, preset.key)}
                </button>
              ))}
            <div className="px-3 py-1.5 text-xs text-muted-foreground mt-1">
              {t("group.previous")}
            </div>
            {presets
              .filter((p) =>
                (["lastMonth", "lastQuarter", "lastYear"] as PresetKey[]).includes(
                  p.key,
                ),
              )
              .map((preset) => (
                <button
                  key={preset.key}
                  className={`text-left px-3 py-1.5 text-sm hover:bg-accent cursor-pointer rounded-sm mx-1 ${
                    selection.type === "preset" && selection.key === preset.key
                      ? "bg-accent font-medium"
                      : ""
                  }`}
                  onClick={() => handlePresetClick(preset)}
                >
                  {formatPresetLabel(t, preset.key)}
                </button>
              ))}
            <div className="h-px bg-border my-1" />
            <button
              className={`text-left px-3 py-1.5 text-sm hover:bg-accent cursor-pointer rounded-sm mx-1 ${
                selection.type === "custom" ? "bg-accent font-medium" : ""
              }`}
              onClick={() => setShowCustom(true)}
            >
              {t("customRange")}
            </button>
          </div>
        )}
      </PopoverContent>
    </Popover>
  )
}
