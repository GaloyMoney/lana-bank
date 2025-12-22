"use client"

import { useMemo, useState } from "react"
import { useTranslations } from "next-intl"

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

export const getInitialDateRange = (): DateRange => {
  const today = new Date()
  const oneYearAgo = new Date(today.getFullYear() - 1, today.getMonth(), today.getDate())
  return {
    from: toDateString(oneYearAgo),
    until: toDateString(today),
  }
}

export const DateRangeSelector = ({
  initialDateRange,
  onDateChange,
}: DateRangeSelectorProps) => {
  const t = useTranslations("DateRangePicker")
  const [isOpen, setIsOpen] = useState(false)
  const [selectedFrom, setSelectedFrom] = useState<Date | undefined>(
    new Date(initialDateRange.from),
  )
  const [selectedTo, setSelectedTo] = useState<Date | undefined>(
    new Date(initialDateRange.until),
  )

  const today = useMemo(() => {
    const date = new Date()
    date.setHours(0, 0, 0, 0)
    return date
  }, [])

  const handleSubmit = () => {
    if (selectedFrom && selectedTo) {
      onDateChange({
        from: toDateString(selectedFrom),
        until: toDateString(selectedTo),
      })
      setIsOpen(false)
    }
  }

  return (
    <Popover open={isOpen} onOpenChange={setIsOpen}>
      <PopoverTrigger asChild>
        <div className="rounded-md bg-input-text p-2 px-4 text-sm border cursor-pointer bg-muted">
          {selectedFrom && selectedTo
            ? `${formatDate(selectedFrom, { includeTime: false })} - ${formatDate(selectedTo, { includeTime: false })}`
            : t("pickDateRange")}
        </div>
      </PopoverTrigger>
      <PopoverContent className="w-auto p-0" align="start">
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
          <div className="border-t p-2 flex justify-end">
            <Button
              onClick={handleSubmit}
              variant="ghost"
              disabled={!selectedFrom || !selectedTo}
            >
              {t("apply")}
            </Button>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  )
}
