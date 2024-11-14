"use client"

import { useState, useEffect } from "react"

import {
  HiChevronUp,
  HiChevronDown,
  HiSelector,
  HiChevronLeft,
  HiChevronRight,
  HiFilter,
} from "react-icons/hi"

import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuCheckboxItem,
} from "@/components/primitive/dropdown-menu"
import { Button } from "@/components/primitive/button"
import { Skeleton } from "@/components/primitive/skeleton"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"

export type Column<T> = {
  [K in keyof T]: {
    key: K
    label: string
    sortable?: boolean
    filterValues?: Array<T[K]>
    render?: (value: T[K], record: T) => React.ReactNode
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
}: PaginatedTableProps<T>): React.ReactElement => {
  const [sortState, setSortState] = useState<{
    column: keyof T | null
    direction: "ASC" | "DESC" | null
  }>({ column: null, direction: null })

  const [filterState, setFilterState] = useState<Partial<Record<keyof T, T[keyof T]>>>({})
  const [currentPage, setCurrentPage] = useState(1)
  const [displayData, setDisplayData] = useState<{ node: T }[]>([])

  useEffect(() => {
    const startIdx = (currentPage - 1) * pageSize
    const endIdx = startIdx + pageSize
    data && setDisplayData(data.edges.slice(startIdx, endIdx))
  }, [data, currentPage, pageSize])

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
    setCurrentPage((prevPage) => prevPage + 1)
  }

  const handlePreviousPage = () => {
    if (currentPage > 1) {
      setCurrentPage((prevPage) => prevPage - 1)
    }
  }

  if (data?.edges.length === 0 && Object.keys(filterState).length === 0) {
    return <div className="text-sm">No data to display</div>
  }

  return (
    <>
      <div className="overflow-x-auto">
        <Table className="table-fixed w-full">
          {showHeader && (
            <TableHeader>
              <TableRow>
                {columns.map((col) => (
                  <TableHead key={col.key as string}>
                    <div className="flex items-center space-x-2 justify-between">
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
                              All
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
              </TableRow>
            </TableHeader>
          )}
          <TableBody>
            {loading
              ? // Render loading skeleton rows while keeping headers visible
                Array.from({ length: displayData.length || pageSize }).map(
                  (_, rowIndex) => (
                    <TableRow key={rowIndex}>
                      {columns.map((_, colIndex) => (
                        <TableCell key={colIndex}>
                          <Skeleton className="h-4 w-full" />
                        </TableCell>
                      ))}
                    </TableRow>
                  ),
                )
              : displayData.map(({ node }, idx) => (
                  <TableRow
                    key={idx}
                    onClick={() => onClick?.(node)}
                    className={onClick ? "cursor-pointer" : ""}
                  >
                    {columns.map((col) => (
                      <TableCell
                        key={col.key as string}
                        className="whitespace-normal break-words"
                      >
                        {col.render
                          ? col.render(node[col.key], node)
                          : String(node[col.key])}
                      </TableCell>
                    ))}
                  </TableRow>
                ))}
          </TableBody>
        </Table>
      </div>

      <div className="flex items-center justify-end space-x-2 py-4">
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
    </>
  )
}

export default PaginatedTable

export const DEFAULT_PAGESIZE = 10
