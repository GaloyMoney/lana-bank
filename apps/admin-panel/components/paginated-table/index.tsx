import React, { useState, useEffect, useRef } from "react"
import Link from "next/link"
import { ArrowRight } from "lucide-react"
import {
  HiChevronUp,
  HiChevronDown,
  HiSelector,
  HiChevronLeft,
  HiChevronRight,
  HiFilter,
} from "react-icons/hi"

import { useRouter } from "next/navigation"

import { Card } from "@lana/web/ui/card"
import { Separator } from "@lana/web/ui/separator"
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuCheckboxItem,
} from "@lana/web/ui/dropdown-menu"
import { Button } from "@lana/web/ui/button"
import { Skeleton } from "@lana/web/ui/skeleton"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@lana/web/ui/table"

import { useBreakpointDown } from "@lana/web/hooks"
import { useTranslations } from "next-intl"

import { getSafeInternalPath } from "@/lib/utils"

export type Column<T> = {
  [K in keyof T]: {
    key: K
    label: string
    sortable?: boolean
    filterValues?: Array<T[K]>
    render?: (value: T[K], record: T) => React.ReactNode
    labelClassName?: string
  }
}[keyof T]

interface PageInfo {
  endCursor: string
  startCursor: string
  hasNextPage: boolean
  hasPreviousPage: boolean
}

export interface PaginatedData<T> {
  edges: { node: T; cursor: string }[]
  pageInfo: PageInfo
}

interface PaginatedTableProps<T> {
  columns: Column<T>[]
  loading: boolean
  data?: PaginatedData<T>
  pageSize: number
  fetchMore: (cursor: string) => Promise<unknown>
  onSort?: (column: keyof T, sortDirection: "ASC" | "DESC") => void
  onFilter?: (column: keyof T, value: T[keyof T] | undefined) => void
  onClick?: (record: T) => void
  showHeader?: boolean
  navigateTo?: (record: T) => string | null
  customFooter?: React.ReactNode
  style?: "compact" | "comfortable"
  noDataText?: string
  subRows?: (record: T) => T[]
  initialPage?: number
  onPageChange?: (page: number) => void
}

const PaginatedTable = <T,>({
  columns,
  loading,
  data,
  pageSize,
  fetchMore,
  onSort,
  onFilter,
  onClick,
  showHeader = true,
  navigateTo,
  customFooter,
  style = "comfortable",
  noDataText,
  subRows,
  initialPage = 1,
  onPageChange,
}: PaginatedTableProps<T>): React.ReactElement => {
  const isMobile = useBreakpointDown("md")
  const t = useTranslations("PaginatedTable")
  const tableRef = useRef<HTMLDivElement>(null)
  const [focusedRowIndex, setFocusedRowIndex] = useState<number>(-1)
  const [isTableFocused, setIsTableFocused] = useState(false)
  const router = useRouter()

  const getSafeNavigationUrl = (item: T): string | null => {
    if (!navigateTo) return null
    return getSafeInternalPath(navigateTo(item))
  }

  const [sortState, setSortState] = useState<{
    column: keyof T | null
    direction: "ASC" | "DESC" | null
  }>({ column: null, direction: null })

  const [filterState, setFilterState] = useState<Partial<Record<keyof T, T[keyof T]>>>({})
  const [currentPage, setCurrentPage] = useState(initialPage)
  const [displayData, setDisplayData] = useState<{ node: T }[]>([])

  useEffect(() => {
    setCurrentPage(initialPage)
  }, [initialPage])

  useEffect(() => {
    const startIdx = (currentPage - 1) * pageSize
    const endIdx = startIdx + pageSize
    data && setDisplayData(data.edges.slice(startIdx, endIdx))
  }, [data, currentPage, pageSize])

  const focusRow = (index: number) => {
    if (index < 0 || !displayData.length || !isTableFocused) return
    const validIndex = Math.min(Math.max(0, index), displayData.length - 1)
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
      if (!displayData.length) return

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
            const node = displayData[focusedRowIndex].node
            if (onClick) {
              onClick(node)
            } else if (navigateTo) {
              const url = getSafeNavigationUrl(node)
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
  }, [displayData, focusedRowIndex, onClick, navigateTo, router, isTableFocused])

  useEffect(() => {
    setFocusedRowIndex(-1)
  }, [currentPage])

  const handleSort = (columnKey: keyof T) => {
    let newDirection: "ASC" | "DESC" = "ASC"
    if (sortState.column === columnKey && sortState.direction === "ASC") {
      newDirection = "DESC"
    }
    setSortState({ column: columnKey, direction: newDirection })
    onSort && onSort(columnKey, newDirection)
  }

  const handleFilter = (columnKey: keyof T, value: T[keyof T] | undefined) => {
    setFilterState({ [columnKey]: value } as Partial<Record<keyof T, T[keyof T]>>)
    onFilter && onFilter(columnKey, value)
  }

  const handleNextPage = async () => {
    const totalDataLoaded = data?.edges.length || 0
    const maxDataRequired = currentPage * pageSize + pageSize

    if (totalDataLoaded < maxDataRequired && data && data.pageInfo.hasNextPage) {
      await fetchMore(data.pageInfo.endCursor)
    }
    const newPage = currentPage + 1
    setCurrentPage(newPage)
    onPageChange?.(newPage)
  }

  const handlePreviousPage = () => {
    if (currentPage > 1) {
      const newPage = currentPage - 1
      setCurrentPage(newPage)
      onPageChange?.(newPage)
    }
  }

  if (loading && !data) {
    return isMobile ? (
      <div className="space-y-4" data-testid="loading-skeleton">
        {Array.from({ length: pageSize }).map((_, idx) => (
          <Card key={idx} className="p-4 space-y-3">
            {columns.map((_, colIndex) => (
              <Skeleton key={colIndex} className="h-4 w-full" />
            ))}
          </Card>
        ))}
      </div>
    ) : (
      <div
        className={`overflow-x-auto rounded-md ${style === "comfortable" ? "border" : ""}`}
      >
        <Table className="table-fixed w-full">
          <TableHeader
            className={
              style === "comfortable" ? "bg-secondary [&_tr:hover]:bg-secondary!" : ""
            }
          >
            <TableRow>
              {columns.map((col, colIdx) => (
                <TableHead
                  className={col.labelClassName}
                  key={`${col.key as string}-${colIdx}`}
                >
                  {col.label}
                </TableHead>
              ))}
              {navigateTo && <TableHead className="w-24" />}
            </TableRow>
          </TableHeader>
          <TableBody>
            {Array.from({ length: pageSize }).map((_, rowIndex) => (
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
        <Separator />
        <div className="flex items-center justify-end space-x-4 py-2 mr-2">
          <Button variant="outline" size="sm" disabled>
            <HiChevronLeft className="h-4 w-4" />
          </Button>
          <div className="flex items-center gap-1">
            <span className="text-sm font-medium">1</span>
          </div>
          <Button variant="outline" size="sm" disabled>
            <HiChevronRight className="h-4 w-4" />
          </Button>
        </div>
      </div>
    )
  }

  if (data?.edges.length === 0 && Object.keys(filterState).length === 0) {
    return (
      <div className={`text-sm ${style === "compact" && "mt-2"}`}>
        {noDataText || t("noData")}
      </div>
    )
  }

  if (isMobile) {
    return (
      <div className="space-y-4">
        <div className="flex flex-wrap items-center gap-2">
          {columns.filter((col) => col.sortable).length > 0 && (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="outline" size="sm">
                  {t("sortBy", { defaultMessage: "Sort By" })}
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent>
                {columns
                  .filter((col) => col.sortable)
                  .map((col) => (
                    <DropdownMenuCheckboxItem
                      key={col.key as string}
                      checked={sortState.column === col.key}
                      onCheckedChange={() => handleSort(col.key)}
                    >
                      {col.label}{" "}
                      {sortState.column === col.key &&
                        (sortState.direction === "ASC" ? "↑" : "↓")}
                    </DropdownMenuCheckboxItem>
                  ))}
              </DropdownMenuContent>
            </DropdownMenu>
          )}

          {columns
            .filter((col) => col.filterValues)
            .map((col) => (
              <DropdownMenu key={col.key as string}>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="outline"
                    size="sm"
                    className={filterState[col.key] ? "border-blue-500" : ""}
                  >
                    {col.label}
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent>
                  <DropdownMenuCheckboxItem
                    checked={!filterState[col.key]}
                    onCheckedChange={() => handleFilter(col.key, undefined)}
                  >
                    {t("all", { defaultMessage: "All" })}
                  </DropdownMenuCheckboxItem>
                  {col.filterValues?.map((value, idx) => (
                    <DropdownMenuCheckboxItem
                      key={idx}
                      checked={filterState[col.key] === value}
                      onCheckedChange={() => handleFilter(col.key, value)}
                      className="capitalize"
                    >
                      {String(value).split("_").join(" ").toLowerCase()}
                    </DropdownMenuCheckboxItem>
                  ))}
                </DropdownMenuContent>
              </DropdownMenu>
            ))}
        </div>

        {displayData.map(({ node }, idx) => {
          const safeUrl = getSafeNavigationUrl(node)
          return (
            <>
              <Card key={idx} className="p-4 space-y-3" onClick={() => onClick?.(node)}>
                {columns.map((col, colIdx) => (
                  <div
                    key={`${col.key as string}-${colIdx}`}
                    className="flex justify-between items-start gap-4"
                  >
                    <div className="text-sm font-medium text-muted-foreground">
                      {col.label}
                    </div>
                    <div className="text-sm">
                      {col.render
                        ? col.render(node[col.key], node)
                        : String(node[col.key])}
                    </div>
                  </div>
                ))}
                {safeUrl && (
                  <div className="pt-2">
                    <Button
                      variant="outline"
                      className="w-full flex items-center justify-center"
                      asChild
                    >
                      <Link href={safeUrl}>
                        {t("view", { defaultMessage: "View" })}
                        <ArrowRight className="h-4 w-4" />
                      </Link>
                    </Button>
                  </div>
                )}
              </Card>
              {subRows &&
                subRows(node).length > 0 &&
                subRows(node).map((subRow, subIdx) => {
                  const subRowSafeUrl = getSafeNavigationUrl(subRow)
                  return (
                    <Card
                      key={`sub-row-${idx}-${subIdx}`}
                      className="p-4 space-y-3"
                      onClick={() => onClick?.(subRow)}
                    >
                      {columns.map((col, colIdx) => (
                        <div
                          key={`${col.key as string}-${colIdx}`}
                          className="flex justify-between items-start gap-4"
                        >
                          <div className="text-sm font-medium text-muted-foreground">
                            {col.label}
                          </div>
                          <div className="text-sm">
                            {col.render
                              ? col.render(subRow[col.key], subRow)
                              : String(subRow[col.key])}
                          </div>
                        </div>
                      ))}
                      {subRowSafeUrl && (
                        <div className="pt-2">
                          <Button
                            variant="outline"
                            size="sm"
                            className="w-full flex items-center justify-center"
                            asChild
                          >
                            <Link href={subRowSafeUrl}>
                              {t("view", { defaultMessage: "View" })}
                              <ArrowRight className="h-4 w-4" />
                            </Link>
                          </Button>
                        </div>
                      )}
                    </Card>
                  )
                })}
            </>
          )
        })}
        <div className="flex items-center justify-end space-x-4 py-2 mr-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handlePreviousPage}
            disabled={currentPage === 1}
          >
            <HiChevronLeft className="h-4 w-4" />
          </Button>
          <div className="flex items-center gap-1">
            <span className="text-sm font-medium">{currentPage}</span>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={handleNextPage}
            disabled={displayData.length < pageSize && !data?.pageInfo.hasNextPage}
          >
            <HiChevronRight className="h-4 w-4" />
          </Button>
        </div>
      </div>
    )
  }

  return (
    <>
      <div
        ref={tableRef}
        className={`overflow-x-auto rounded-md focus:outline-none ${style === "comfortable" ? "border" : ""}`}
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
        <Table className="table-fixed w-full">
          {showHeader && (
            <TableHeader
              className={
                style === "comfortable" ? "bg-secondary [&_tr:hover]:bg-secondary!" : ""
              }
            >
              <TableRow>
                {columns.map((col, colIdx) => (
                  <TableHead
                    key={`${col.key as string}-${colIdx}`}
                    className={col.labelClassName}
                  >
                    <div className="flex items-center gap-2">
                      <span>{col.label}</span>
                      {col.sortable && (
                        <Button
                          variant="ghost"
                          size="sm"
                          className="h-8 w-8 p-0"
                          onClick={() => handleSort(col.key)}
                        >
                          {sortState.column === col.key ? (
                            sortState.direction === "ASC" ? (
                              <HiChevronUp className="h-4 w-4 text-blue-500" />
                            ) : (
                              <HiChevronDown className="h-4 w-4 text-blue-500" />
                            )
                          ) : (
                            <HiSelector className="h-4 w-4" />
                          )}
                        </Button>
                      )}
                      {col.filterValues && (
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                              <HiFilter
                                className={`h-4 w-4 ${
                                  filterState[col.key] ? "text-blue-500" : ""
                                }`}
                              />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent>
                            <DropdownMenuCheckboxItem
                              checked={!filterState[col.key]}
                              onCheckedChange={() => handleFilter(col.key, undefined)}
                            >
                              {t("all", { defaultMessage: "All" })}
                            </DropdownMenuCheckboxItem>
                            {col.filterValues.map((value, idx) => (
                              <DropdownMenuCheckboxItem
                                key={idx}
                                checked={filterState[col.key] === value}
                                onCheckedChange={() => handleFilter(col.key, value)}
                                className="capitalize"
                              >
                                {String(value).split("_").join(" ").toLowerCase()}
                              </DropdownMenuCheckboxItem>
                            ))}
                          </DropdownMenuContent>
                        </DropdownMenu>
                      )}
                    </div>
                  </TableHead>
                ))}
                {navigateTo && <TableHead className="w-24"></TableHead>}
              </TableRow>
            </TableHeader>
          )}
          <TableBody>
            {displayData.map(({ node }, idx) => {
              const safeUrl = getSafeNavigationUrl(node)
              return (
                <React.Fragment key={idx}>
                  <TableRow
                    data-testid={`table-row-${idx}`}
                    onClick={() => onClick?.(node)}
                    tabIndex={0}
                    className={`${onClick ? "cursor-pointer" : ""} ${
                      focusedRowIndex === idx ? "bg-muted" : ""
                    } hover:bg-muted/50 transition-colors outline-none`}
                    onFocus={() => setFocusedRowIndex(idx)}
                    role="row"
                    aria-selected={focusedRowIndex === idx}
                  >
                    {columns.map((col, colIdx) => (
                      <TableCell
                        key={`${col.key as string}-${colIdx}`}
                        className={
                          style === "comfortable"
                            ? "whitespace-normal wrap-break-word"
                            : ""
                        }
                      >
                        {col.render
                          ? col.render(node[col.key], node)
                          : String(node[col.key])}
                      </TableCell>
                    ))}
                    {navigateTo && (
                      <TableCell>
                        {safeUrl && (
                          <Button
                            variant="outline"
                            size="sm"
                            className="w-full flex items-center"
                            asChild
                          >
                            <Link href={safeUrl}>
                              {t("view")}
                              <ArrowRight />
                            </Link>
                          </Button>
                        )}
                      </TableCell>
                    )}
                  </TableRow>
                  {subRows &&
                    subRows(node).length > 0 &&
                    subRows(node).map((subRow, subIdx) => {
                      const subRowSafeUrl = getSafeNavigationUrl(subRow)
                      return (
                        <TableRow
                          role="row"
                          key={`sub-row-${idx}-${subIdx}`}
                          onClick={() => onClick?.(subRow)}
                          tabIndex={0}
                          className={`${onClick ? "cursor-pointer" : ""} ${
                            focusedRowIndex === idx ? "bg-muted" : ""
                          } hover:bg-muted/50 transition-colors outline-none`}
                        >
                          {columns.map((col, colIdx) => (
                            <TableCell
                              key={`${col.key as string}-${colIdx}`}
                              className={
                                style === "comfortable"
                                  ? "whitespace-normal wrap-break-word"
                                  : ""
                              }
                            >
                              {col.render
                                ? col.render(subRow[col.key], subRow)
                                : String(subRow[col.key])}
                            </TableCell>
                          ))}
                          {navigateTo && (
                            <TableCell>
                              {subRowSafeUrl && (
                                <Button
                                  variant="outline"
                                  size="sm"
                                  className="w-full flex items-center"
                                  asChild
                                >
                                  <Link href={subRowSafeUrl}>
                                    {t("view")}
                                    <ArrowRight />
                                  </Link>
                                </Button>
                              )}
                            </TableCell>
                          )}
                        </TableRow>
                      )
                    })}
                </React.Fragment>
              )
            })}
          </TableBody>
          {customFooter}
        </Table>
        <Separator />
        <div className="flex items-center justify-end space-x-4 py-2 mr-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handlePreviousPage}
            disabled={currentPage === 1}
          >
            <HiChevronLeft className="h-4 w-4" />
          </Button>
          <div className="flex items-center gap-1">
            <span className="text-sm font-medium">{currentPage}</span>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={handleNextPage}
            disabled={
              !loading && displayData.length < pageSize && !data?.pageInfo.hasNextPage
            }
          >
            <HiChevronRight className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </>
  )
}

export default PaginatedTable

export const DEFAULT_PAGESIZE = 10
