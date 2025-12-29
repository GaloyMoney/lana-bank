"use client"

import React, { useState, useEffect, useRef } from "react"
import Link from "next/link"
import { ArrowRight } from "lucide-react"
import { useRouter } from "next/navigation"

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@lana/web/ui/table"
import { Button } from "@lana/web/ui/button"
import { Skeleton } from "@lana/web/ui/skeleton"
import { Card } from "@lana/web/ui/card"
import { useBreakpointDown } from "@lana/web/hooks"

import { useTranslations } from "next-intl"

import { cn, getSafeInternalPath } from "@/lib/utils"

export type Column<T> = {
  [K in keyof T]: {
    key: K
    header: string | React.ReactNode
    width?: string
    align?: "left" | "center" | "right"
    render?: (value: T[K], record: T, index: number) => React.ReactNode
  }
}[keyof T]

interface DataTableProps<T> {
  data: T[]
  columns: Column<T>[]
  className?: string
  headerClassName?: string
  rowClassName?: string | ((item: T, index: number) => string)
  cellClassName?: string | ((column: Column<T>, item: T) => string)
  onRowClick?: (item: T) => void
  emptyMessage?: React.ReactNode
  loading?: boolean
  navigateTo?: (record: T) => string | null
}

const DataTable = <T,>({
  data,
  columns,
  className,
  headerClassName,
  rowClassName,
  cellClassName,
  onRowClick,
  emptyMessage,
  loading = false,
  navigateTo,
}: DataTableProps<T>) => {
  const t = useTranslations("DataTable")
  const isMobile = useBreakpointDown("md")
  const [focusedRowIndex, setFocusedRowIndex] = useState<number>(-1)
  const [isTableFocused, setIsTableFocused] = useState(false)
  const tableRef = useRef<HTMLDivElement>(null)
  const router = useRouter()

  const getSafeNavigationUrl = (item: T): string | null => {
    if (!navigateTo) return null
    return getSafeInternalPath(navigateTo(item))
  }

  const focusRow = (index: number) => {
    if (index < 0 || !data.length || !isTableFocused) return
    const validIndex = Math.min(Math.max(0, index), data.length - 1)
    const row = document.querySelector(
      `[data-testid="table-row-${validIndex}"]`,
    ) as HTMLElement
    if (row) {
      row.focus({ preventScroll: true })
      row.scrollIntoView({ behavior: "smooth", block: "nearest" })
      setFocusedRowIndex(validIndex)
    }
  }

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!tableRef.current?.contains(document.activeElement) || !isTableFocused) return
      if (
        document.activeElement?.tagName === "INPUT" ||
        document.activeElement?.tagName === "TEXTAREA" ||
        document.activeElement?.tagName === "SELECT" ||
        document.activeElement?.tagName === "BUTTON"
      )
        return
      if (!data.length) return

      switch (e.key) {
        case "ArrowUp":
          e.preventDefault()
          focusRow(focusedRowIndex - 1)
          break
        case "ArrowDown":
          e.preventDefault()
          focusRow(focusedRowIndex + 1)
          break
        case "Enter":
          e.preventDefault()
          if (focusedRowIndex >= 0) {
            const item = data[focusedRowIndex]
            if (onRowClick) {
              onRowClick(item)
            } else if (navigateTo) {
              const url = getSafeNavigationUrl(item)
              if (url) {
                router.push(url)
              }
            }
          }
          break
      }
    }

    if (isTableFocused) {
      window.addEventListener("keydown", handleKeyDown)
      return () => window.removeEventListener("keydown", handleKeyDown)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data, focusedRowIndex, onRowClick, navigateTo, isTableFocused])

  if (loading && !data.length) {
    return isMobile ? (
      <div className="space-y-4" data-testid="loading-skeleton">
        {Array.from({ length: 5 }).map((_, idx) => (
          <Card key={idx} className="p-4 space-y-3">
            {columns.map((_, colIndex) => (
              <Skeleton key={colIndex} className="h-4 w-full" />
            ))}
          </Card>
        ))}
      </div>
    ) : (
      <div className="overflow-x-auto border rounded-md" data-testid="loading-skeleton">
        <Table className={cn("table-fixed w-full", className)}>
          <TableHeader className="bg-secondary [&_tr:hover]:bg-secondary!">
            <TableRow className={headerClassName}>
              {columns.map((column, index) => (
                <TableHead
                  key={index}
                  className={cn(
                    column.align === "center" && "text-center",
                    column.align === "right" && "text-right",
                  )}
                  style={{ width: column.width }}
                >
                  {column.header}
                </TableHead>
              ))}
              {navigateTo && <TableHead className="w-24" />}
            </TableRow>
          </TableHeader>
          <TableBody>
            {Array.from({ length: 5 }).map((_, rowIndex) => (
              <TableRow key={rowIndex}>
                {columns.map((_, colIndex) => (
                  <TableCell key={colIndex}>
                    <Skeleton className="h-9 w-full" />
                  </TableCell>
                ))}
                {navigateTo && (
                  <TableCell>
                    <Skeleton className="h-9 w-full" />
                  </TableCell>
                )}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    )
  }

  if (!data.length) {
    return <div className="text-sm">{emptyMessage || t("noData")}</div>
  }

  if (isMobile) {
    return (
      <div className="space-y-4">
        {data.map((item, index) => {
          const safeNavigationUrl = getSafeNavigationUrl(item)
          return (
            <Card
              key={index}
              className={cn(
                "p-4 space-y-3",
                typeof rowClassName === "function"
                  ? rowClassName(item, index)
                  : rowClassName,
                onRowClick && "cursor-pointer",
              )}
              onClick={() => onRowClick?.(item)}
            >
              {columns.map((column, colIndex) => {
                const hasHeader =
                  typeof column.header === "string" && column.header.trim() !== ""
                return (
                  <div
                    key={colIndex}
                    className={cn(
                      "flex items-start gap-4",
                      hasHeader ? "justify-between" : "w-full",
                      typeof cellClassName === "function"
                        ? cellClassName(column, item)
                        : cellClassName,
                    )}
                  >
                    {hasHeader && (
                      <div className="text-sm font-medium text-muted-foreground">
                        {column.header}
                      </div>
                    )}
                    <div
                      className={cn(
                        "text-sm",
                        !hasHeader && "w-full",
                        column.align === "center" && "text-center",
                        column.align === "right" && "text-right",
                      )}
                    >
                      {column.render
                        ? column.render(item[column.key], item, index)
                        : String(item[column.key])}
                    </div>
                  </div>
                )
              })}
              {safeNavigationUrl && (
                <div className="pt-2">
                  <Link href={safeNavigationUrl}>
                    <Button
                      variant="outline"
                      size="sm"
                      className="w-full flex items-center justify-center"
                    >
                      {t("view")}
                      <ArrowRight className="h-4 w-4" />
                    </Button>
                  </Link>
                </div>
              )}
            </Card>
          )
        })}
      </div>
    )
  }

  return (
    <div
      ref={tableRef}
      className="overflow-x-auto border rounded-md focus:outline-none"
      tabIndex={0}
      role="grid"
      onFocus={() => setIsTableFocused(true)}
      onBlur={(e) => {
        if (!tableRef.current?.contains(e.relatedTarget as Node)) {
          setIsTableFocused(false)
          setFocusedRowIndex(-1)
        }
      }}
    >
      <Table className={cn("table-fixed w-full", className)}>
        <TableHeader className="bg-secondary [&_tr:hover]:bg-secondary!">
          <TableRow className={headerClassName}>
            {columns.map((column, index) => (
              <TableHead
                key={index}
                className={cn(
                  column.align === "center" && "text-center",
                  column.align === "right" && "text-right",
                )}
                style={{ width: column.width }}
              >
                {column.header}
              </TableHead>
            ))}
            {navigateTo && <TableHead className="w-24" />}
          </TableRow>
        </TableHeader>
        <TableBody>
          {data.map((item, rowIndex) => {
            const safeNavigationUrl = getSafeNavigationUrl(item)
            return (
              <TableRow
                data-testid={`table-row-${rowIndex}`}
                key={rowIndex}
                onClick={() => onRowClick?.(item)}
                tabIndex={0}
                className={cn(
                  typeof rowClassName === "function"
                    ? rowClassName(item, rowIndex)
                    : rowClassName,
                  onRowClick && "cursor-pointer",
                  focusedRowIndex === rowIndex && "bg-muted",
                  "hover:bg-muted/50 transition-colors outline-none",
                )}
                onFocus={() => setFocusedRowIndex(rowIndex)}
                role="row"
                aria-selected={focusedRowIndex === rowIndex}
              >
                {columns.map((column, colIndex) => (
                  <TableCell
                    key={colIndex}
                    className={cn(
                      "whitespace-normal wrap-break-word",
                      column.align === "center" && "text-center",
                      column.align === "right" && "text-right",
                      typeof cellClassName === "function"
                        ? cellClassName(column, item)
                        : cellClassName,
                    )}
                  >
                    {column.render
                      ? column.render(item[column.key], item, rowIndex)
                      : String(item[column.key])}
                  </TableCell>
                ))}
                {safeNavigationUrl && (
                  <TableCell>
                    <Link href={safeNavigationUrl} className="group">
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full flex items-center"
                      >
                        {t("view")}
                        <ArrowRight />
                      </Button>
                    </Link>
                  </TableCell>
                )}
              </TableRow>
            )
          })}
        </TableBody>
      </Table>
    </div>
  )
}

export default DataTable
