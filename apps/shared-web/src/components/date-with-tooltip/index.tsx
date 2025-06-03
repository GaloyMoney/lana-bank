"use client"

import { ReactNode } from "react"

import {
  Tooltip,
  TooltipProvider,
  TooltipTrigger,
  TooltipContent,
} from "@lana/web/ui/tooltip"
import { formatDate } from "@lana/web/utils"

interface DateWithTooltipProps {
  value: string | number | Date
  renderShort?: (short: string) => ReactNode
}

export function DateWithTooltip({ value, renderShort }: DateWithTooltipProps) {
  const short = formatDate(value, { includeTime: false })
  const full = formatDate(value)

  return (
    <TooltipProvider delayDuration={100}>
      <Tooltip>
        <TooltipTrigger asChild>
          <span className="cursor-default">
            {renderShort ? renderShort(short) : short}
          </span>
        </TooltipTrigger>
        <TooltipContent side="top" align="center">
          {full}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )
}

export default DateWithTooltip
