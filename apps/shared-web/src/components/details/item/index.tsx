"use client"

import * as React from "react"

import Link from "next/link"

import { DetailsGroupContext } from "../group"

import { cn } from "@lana/web/utils"

export type DetailItemProps = {
  label: React.ReactNode
  value: React.ReactNode
  className?: string
  onClick?: (() => void) | null
  showHoverEffect?: boolean
  labelTestId?: string
  valueTestId?: string
  keyClassName?: string
  href?: string
}

export const DetailItem: React.FC<DetailItemProps> = ({
  label,
  value,
  className,
  onClick = null,
  showHoverEffect = false,
  labelTestId,
  valueTestId,
  href,
}) => {
  const layout = React.useContext(DetailsGroupContext)

  const styles = {
    container: cn(
      "rounded-md font-semibold flex-wrap",
      layout === "vertical"
        ? "flex flex-col justify-between"
        : "flex justify-between items-center p-1",
      (showHoverEffect || onClick || href) && "hover:bg-secondary",
      className,
    ),
    label: cn("text-muted-foreground", layout === "vertical" ? "text-sm" : "font-normal"),
    value: cn("text-md"),
  }

  const content = (
    <div
      className={styles.container}
      onClick={onClick || undefined}
      data-testid={labelTestId}
    >
      <div className={styles.label}>{label}</div>
      <div className={styles.value} data-testid={valueTestId}>
        {value}
      </div>
    </div>
  )

  if (href) {
    return (
      <Link href={href} className="no-underline hover:no-underline">
        {content}
      </Link>
    )
  }

  return content
}
