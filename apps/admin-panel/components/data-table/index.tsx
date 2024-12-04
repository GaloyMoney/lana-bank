"use client"

import React from "react"

import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/ui/table"
import { cn } from "@/lib/utils"
import { Skeleton } from "@/ui/skeleton"

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
}

const DEFAULT_ROWS = 10

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
}: DataTableProps<T>) => {
  const emptyRowsToFill = DEFAULT_ROWS - (data?.length || 0)

  if (loading) {
    return (
      <div className="w-full overflow-x-auto border rounded-md">
        <Table className={className}>
          <TableHeader>
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
            </TableRow>
          </TableHeader>
          <TableBody>
            {Array.from({ length: DEFAULT_ROWS }).map((_, rowIndex) => (
              <TableRow key={rowIndex}>
                {columns.map((_, colIndex) => (
                  <TableCell key={colIndex}>
                    <Skeleton className="h-5 w-full" />
                  </TableCell>
                ))}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    )
  }

  if (!data.length && emptyMessage) {
    return emptyMessage
  }

  if (!data.length) {
    return (
      <div className="overflow-x-auto border rounded-md">
        <div className="w-full h-[calc(21px*20)] grid place-items-center">
          <div className="text-sm">No data to display</div>
        </div>
      </div>
    )
  }

  return (
    <div className="w-full overflow-x-auto border rounded-md">
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
          </TableRow>
        </TableHeader>
        <TableBody>
          {data.map((item, rowIndex) => (
            <TableRow
              key={rowIndex}
              onClick={() => onRowClick?.(item)}
              className={cn(
                typeof rowClassName === "function"
                  ? rowClassName(item, rowIndex)
                  : rowClassName,
                onRowClick && "cursor-pointer",
              )}
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
            </TableRow>
          ))}
          {emptyRowsToFill > 0 &&
            Array.from({ length: emptyRowsToFill }).map((_, idx) => (
              <TableRow key={`empty-${idx}`} className="border-none hover:bg-transparent">
                {columns.map((_, colIndex) => (
                  <TableCell key={colIndex}>&nbsp;</TableCell>
                ))}
              </TableRow>
            ))}
        </TableBody>
      </Table>
    </div>
  )
}

export default DataTable
