"use client"

import { useMemo, useState } from "react"
import { useTranslations } from "next-intl"

import { Button } from "@lana/web/ui/button"
import { Calendar } from "@lana/web/ui/calendar"
import { Popover, PopoverContent, PopoverTrigger } from "@lana/web/ui/popover"

import { formatDate } from "@lana/web/utils"

import { parseDateString } from "@/components/date-range-picker"

type AsOfDateSelectorProps = {
  asOf: string
  onDateChange: (asOf: string) => void
}

const toDateString = (date: Date): string => {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, "0")
  const day = String(date.getDate()).padStart(2, "0")
  return `${year}-${month}-${day}`
}

export const getInitialAsOfDate = (): string => {
  return toDateString(new Date())
}

export const AsOfDateSelector = ({ asOf, onDateChange }: AsOfDateSelectorProps) => {
  const t = useTranslations("BalanceSheet")
  const [isOpen, setIsOpen] = useState(false)
  const [selectedDate, setSelectedDate] = useState<Date | undefined>(parseDateString(asOf))

  const today = useMemo(() => {
    const date = new Date()
    date.setHours(0, 0, 0, 0)
    return date
  }, [])

  const handleSubmit = () => {
    if (selectedDate) {
      onDateChange(toDateString(selectedDate))
      setIsOpen(false)
    }
  }

  return (
    <Popover open={isOpen} onOpenChange={setIsOpen}>
      <PopoverTrigger asChild>
        <div className="rounded-md bg-input-text p-2 px-4 text-sm border cursor-pointer bg-muted">
          {selectedDate ? formatDate(selectedDate, { includeTime: false }) : t("asOf")}
        </div>
      </PopoverTrigger>
      <PopoverContent className="w-auto p-0" align="start">
        <div className="flex flex-col">
          <Calendar
            mode="single"
            selected={selectedDate}
            onSelect={setSelectedDate}
            defaultMonth={selectedDate}
            disabled={(date) => date > today}
            captionLayout="dropdown"
          />
          <div className="border-t p-2 flex justify-end">
            <Button onClick={handleSubmit} variant="ghost" disabled={!selectedDate}>
              {t("apply")}
            </Button>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  )
}
