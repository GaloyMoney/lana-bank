import React, { useState, useCallback } from "react"

import { Input } from "../primitive/input"
import { Button } from "../primitive/button"
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "../primitive/dialog"
import { Label } from "../primitive/label"

type DateRangeSelectorProps = {
  initialDateRange: DateRange
  onDateChange: (dateRange: DateRange) => void
}

export type DateRange = {
  from: string
  until: string
}

export const getInitialDateRange = (): DateRange => {
  const today = new Date()
  const oneYearAgo = new Date(today.getFullYear() - 1, today.getMonth(), today.getDate())
  return {
    from: oneYearAgo.toISOString(),
    until: today.toISOString(),
  }
}

export const DateRangeSelector: React.FC<DateRangeSelectorProps> = ({
  initialDateRange,
  onDateChange,
}) => {
  const [isOpen, setIsOpen] = useState(false)
  const [startDate, setStartDate] = useState(() => initialDateRange.from.split("T")[0])
  const [endDate, setEndDate] = useState(() => initialDateRange.until.split("T")[0])
  const [error, setError] = useState("")
  const [displayRange, setDisplayRange] = useState(`${startDate} - ${endDate}`)

  const updateDateRange = useCallback(() => {
    if (startDate && endDate && !error) {
      const newDateRange: DateRange = {
        from: new Date(`${startDate}T00:00:00Z`).toISOString(),
        until: new Date(`${endDate}T23:59:59Z`).toISOString(),
      }
      onDateChange(newDateRange)
      setDisplayRange(`${startDate} - ${endDate}`)
    }
  }, [startDate, endDate, error, onDateChange])

  const handleStartDateChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newStartDate = e.target.value
    setStartDate(newStartDate)
    if (newStartDate > endDate) {
      setEndDate(newStartDate)
    }
    setError("")
  }

  const handleEndDateChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newEndDate = e.target.value
    if (newEndDate >= startDate) {
      setEndDate(newEndDate)
      setError("")
    } else {
      setError("End date cannot be earlier than start date")
    }
  }

  const handleSubmit = () => {
    updateDateRange()
    setIsOpen(false)
  }

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>
        <div className="rounded-md bg-input-text p-2 px-4 text-sm border cursor-pointer">
          {displayRange}
        </div>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Select Date Range</DialogTitle>
        </DialogHeader>
        <div className="flex gap-2 w-full">
          <div className="w-1/2">
            <Label htmlFor="start-date" className="text-right">
              Start Date
            </Label>
            <Input
              id="start-date"
              type="date"
              value={startDate}
              onChange={handleStartDateChange}
              max={new Date().toISOString().split("T")[0]}
              className="text-white caret-white appearance-none [&::-webkit-calendar-picker-indicator]:invert"
            />
          </div>
          <div className="w-1/2">
            <Label htmlFor="end-date" className="text-right">
              End Date
            </Label>
            <Input
              id="end-date"
              type="date"
              value={endDate}
              onChange={handleEndDateChange}
              min={startDate}
              max={new Date().toISOString().split("T")[0]}
              className="text-white caret-white appearance-none [&::-webkit-calendar-picker-indicator]:invert"
            />
          </div>
        </div>

        {error && <p className="text-destructive">{error}</p>}
        <DialogFooter>
          <Button onClick={handleSubmit} disabled={!!error} className="w-full mt-4">
            Apply
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
