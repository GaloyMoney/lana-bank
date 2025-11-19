"use client"

import React, { useEffect, useRef, useState } from "react"

import { Tooltip, TooltipContent, TooltipTrigger } from "@lana/web/ui/tooltip"

type TruncatedTextCellProps = {
  children: React.ReactNode
  tooltipText: string
  className?: string
}

export const TruncatedTextCell: React.FC<TruncatedTextCellProps> = ({
  children,
  tooltipText,
  className = "",
}) => {
  const [isTruncated, setIsTruncated] = useState(false)
  const textRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const checkTruncation = () => {
      const element = textRef.current
      if (!element) return

      const isTruncatedHorizontally = element.scrollWidth > element.clientWidth
      const isTruncatedVertically = element.scrollHeight > element.clientHeight
      setIsTruncated(isTruncatedHorizontally || isTruncatedVertically)
    }

    checkTruncation()
    window.addEventListener("resize", checkTruncation)
    return () => window.removeEventListener("resize", checkTruncation)
  }, [tooltipText])

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <div
          ref={textRef}
          className={`truncate overflow-hidden cursor-default ${className}`}
        >
          {children}
        </div>
      </TooltipTrigger>
      {isTruncated && <TooltipContent>{tooltipText}</TooltipContent>}
    </Tooltip>
  )
}
