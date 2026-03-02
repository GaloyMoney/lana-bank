"use client"

import { useEffect, useMemo, useState } from "react"
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

export const getInitialAsOfDate = (): string => toDateString(new Date())

export const AsOfDateSelector = ({ asOf, onDateChange }: AsOfDateSelectorProps) => {
  const t = useTranslations("BalanceSheet")
  const appliedDate = useMemo(() => parseDateString(asOf), [asOf])
  const [isOpen, setIsOpen] = useState(false)
  const [selectedDate, setSelectedDate] = useState<Date | undefined>(appliedDate)

  const today = useMemo(() => {
    const date = new Date()
    date.setHours(0, 0, 0, 0)
    return date
  }, [])

  useEffect(() => {
    setSelectedDate(appliedDate)
  }, [appliedDate])

  const handleSubmit = () => {
    if (selectedDate) {
      onDateChange(toDateString(selectedDate))
      setIsOpen(false)
    }
  }

  const handleOpenChange = (open: boolean) => {
    setIsOpen(open)
    if (!open) {
      setSelectedDate(appliedDate)
    }
  }

  return (
    <Popover open={isOpen} onOpenChange={handleOpenChange}>
      <PopoverTrigger asChild>
        <div className="cursor-pointer rounded-md border bg-muted p-2 px-4 text-sm">
          {formatDate(appliedDate, { includeTime: false })}
        </div>
      </PopoverTrigger>
      <PopoverContent align="start" className="w-auto p-0">
        <div className="flex flex-col">
          <Calendar
            captionLayout="dropdown"
            defaultMonth={selectedDate}
            disabled={(date) => date > today}
            mode="single"
            onSelect={setSelectedDate}
            selected={selectedDate}
          />
          <div className="flex justify-end border-t p-2">
            <Button disabled={!selectedDate} onClick={handleSubmit} variant="ghost">
              {t("apply")}
            </Button>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  )
}
