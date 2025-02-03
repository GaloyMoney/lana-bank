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

import { cn } from "@/lib/utils"

export type Column<T> = {
  [K in keyof T]: {
    key: K
    header: string | React.ReactNode
    width?: string
    align?: "left" | "center" | "right"
    render?: (value: T[K], record: T) => React.ReactNode
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
  emptyMessage = "No data to display",
  loading = false,
  navigateTo,
}: DataTableProps<T>) => {
  const isMobile = useBreakpointDown("md")
  const [focusedRowIndex, setFocusedRowIndex] = useState<number>(-1)
  const [isTableFocused, setIsTableFocused] = useState(false)
  const tableRef = useRef<HTMLDivElement>(null)
  const focusTimeoutRef = useRef<NodeJS.Timeout>()
  const router = useRouter()

  const getNavigationUrl = (item: T): string | null => {
    return navigateTo ? navigateTo(item) : null
  }

  const shouldShowNavigation = (item: T): boolean => {
    if (!navigateTo) return false
    const url = getNavigationUrl(item)
    return url !== null && url !== ""
  }

  const isNoFocusActive = () => {
    const activeElement = document.activeElement
    const isBaseElement =
      !activeElement ||
      activeElement === document.body ||
      activeElement === document.documentElement
    const isOutsideTable = !tableRef.current?.contains(activeElement)
    const isInteractiveElement = activeElement?.matches(
      "button, input, select, textarea, a[href], [tabindex], [contenteditable]",
    )
    return (isBaseElement || isOutsideTable) && !isInteractiveElement
  }

  const smartFocus = () => {
    if (isNoFocusActive()) {
      if (focusTimeoutRef.current) {
        clearTimeout(focusTimeoutRef.current)
      }

      focusTimeoutRef.current = setTimeout(() => {
        if (tableRef.current) {
          tableRef.current.focus()
          setIsTableFocused(true)

          const targetIndex = focusedRowIndex >= 0 ? focusedRowIndex : 0
          const targetRow = document.querySelector(
            `[data-testid="table-row-${targetIndex}"]`,
          ) as HTMLElement

          if (targetRow) {
            targetRow.focus()
            setFocusedRowIndex(targetIndex)
          }
        }
      }, 0)
    }
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
              const url = getNavigationUrl(item)
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

  useEffect(() => {
    const shouldAutoFocus = data && data.length > 0 && !loading
    if (shouldAutoFocus) {
      smartFocus()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data?.length, loading])

  useEffect(() => {
    const handleFocusOut = (e: FocusEvent) => {
      if (!tableRef.current?.contains(e.relatedTarget as Node)) {
        if (isNoFocusActive()) {
          smartFocus()
        }
      }
    }

    document.addEventListener("focusout", handleFocusOut)
    return () => document.removeEventListener("focusout", handleFocusOut)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  useEffect(() => {
    return () => {
      if (focusTimeoutRef.current) {
        clearTimeout(focusTimeoutRef.current)
      }
    }
  }, [])

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
      <div
        className="w-full overflow-x-auto border rounded-md"
        data-testid="loading-skeleton"
      >
        <Table className={className}>
          <TableHeader className="bg-secondary [&_tr:hover]:!bg-secondary">
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
              {navigateTo && <TableHead className="w-24"></TableHead>}
            </TableRow>
          </TableHeader>
          <TableBody>
            {Array.from({ length: 5 }).map((_, rowIndex) => (
              <TableRow key={rowIndex}>
                {columns.map((_, colIndex) => (
                  <TableCell key={colIndex}>
                    <Skeleton className="h-4 w-full" />
                  </TableCell>
                ))}
                {navigateTo && (
                  <TableCell>
                    <Skeleton className="h-4 w-full" />
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
    return <div className="text-sm">{emptyMessage}</div>
  }

  if (isMobile) {
    return (
      <div className="space-y-4">
        {data.map((item, index) => (
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
                      ? column.render(item[column.key], item)
                      : String(item[column.key])}
                  </div>
                </div>
              )
            })}
            {shouldShowNavigation(item) && (
              <div className="pt-2">
                <Link href={getNavigationUrl(item) || ""}>
                  <Button
                    variant="outline"
                    className="w-full flex items-center justify-center"
                  >
                    View
                    <ArrowRight className="h-4 w-4" />
                  </Button>
                </Link>
              </div>
            )}
          </Card>
        ))}
      </div>
    )
  }

  return (
    <div
      ref={tableRef}
      className="w-full overflow-x-auto border rounded-md focus:outline-none"
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
      <Table className={className}>
        <TableHeader className="bg-secondary [&_tr:hover]:!bg-secondary">
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
            {navigateTo && <TableHead className="w-24"></TableHead>}
          </TableRow>
        </TableHeader>
        <TableBody>
          {data.map((item, rowIndex) => (
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
                    column.align === "center" && "text-center",
                    column.align === "right" && "text-right",
                    typeof cellClassName === "function"
                      ? cellClassName(column, item)
                      : cellClassName,
                  )}
                >
                  {column.render
                    ? column.render(item[column.key], item)
                    : String(item[column.key])}
                </TableCell>
              ))}
              {shouldShowNavigation(item) && (
                <TableCell>
                  <Link href={getNavigationUrl(item) || ""}>
                    <Button
                      variant="outline"
                      className="w-full flex items-center justify-between"
                    >
                      View
                      <ArrowRight className="h-4 w-4" />
                    </Button>
                  </Link>
                </TableCell>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}

export default DataTable
