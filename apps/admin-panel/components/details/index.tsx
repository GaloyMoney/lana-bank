import * as React from "react"

import { cn } from "@/lib/utils"

const DetailsGroup = ({
  children,
  className,
}: {
  children: React.ReactNode
  className?: string
}) => {
  return <div className={cn("flex flex-col", className)}>{children}</div>
}

const DetailItem = ({
  label,
  value,
  className,
  onClick = null,
  hover = false,
}: {
  label: React.ReactNode
  value: React.ReactNode
  className?: string
  onClick?: null | (() => void)
  hover?: boolean
}) => {
  const onClickHoverClass = onClick
    ? "hover:cursor-pointer hover:bg-secondary-foreground"
    : ""

  const hoverClass = hover ? "hover:bg-secondary-foreground" : ""

  return (
    <div
      className={cn(
        "flex justify-between items-center p-1 px-2 rounded-md font-semibold flex-wrap",
        className,
        onClickHoverClass,
        hoverClass,
      )}
      onClick={onClick || undefined}
    >
      <div className="text-textColor-secondary font-normal">{label}</div>
      <div>{value}</div>
    </div>
  )
}

export { DetailItem, DetailsGroup }
