"use client"

import { useState, useEffect } from "react"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import { Button } from "@/components/primitive/button"
import {
  HiChevronUp,
  HiChevronDown,
  HiSelector,
  HiChevronLeft,
  HiChevronRight,
} from "react-icons/hi"
import { Select } from "../primitive/select"

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
  data: PaginatedData<T>
  pageSize: number
  fetchMore: (cursor: string) => Promise<unknown>
  onSort?: (column: keyof T, sortDirection: "ASC" | "DESC") => void
  onFilter?: (column: keyof T, value: T[keyof T]) => void
  onClick?: (record: T) => void
}

const PaginatedTable = <T,>({
  columns,
  data,
  pageSize,
  fetchMore,
  onSort,
  onFilter,
  onClick,
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
    setDisplayData(data.edges.slice(startIdx, endIdx))
  }, [data.edges, currentPage, pageSize])

  const handleSort = (columnKey: keyof T) => {
    let newDirection: "ASC" | "DESC" = "ASC"
    if (sortState.column === columnKey && sortState.direction === "ASC") {
      newDirection = "DESC"
    }
    setSortState({ column: columnKey, direction: newDirection })
    onSort && onSort(columnKey, newDirection)
  }

  const handleFilter = (columnKey: keyof T, value: T[keyof T]) => {
    setFilterState((prev) => ({ ...prev, [columnKey]: value }))
    onFilter && onFilter(columnKey, value)
  }

  const handleNextPage = async () => {
    const totalDataLoaded = data.edges.length
    const maxDataRequired = currentPage * pageSize + pageSize

    if (totalDataLoaded < maxDataRequired && data.pageInfo.hasNextPage) {
      await fetchMore(data.pageInfo.endCursor)
    }
    setCurrentPage((prevPage) => prevPage + 1)
  }

  const handlePreviousPage = () => {
    if (currentPage > 1) {
      setCurrentPage((prevPage) => prevPage - 1)
    }
  }

  return (
    <>
      <div>
        <Table>
          <TableHeader>
            <TableRow>
              {columns.map((col) => (
                <TableHead key={col.key as string}>
                  <div className="flex items-center space-x-2">
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
                            <HiChevronUp className="h-4 w-4" />
                          ) : (
                            <HiChevronDown className="h-4 w-4" />
                          )
                        ) : (
                          <HiSelector className="h-4 w-4" />
                        )}
                      </Button>
                    )}
                    {col.filterValues && (
                      <div className="w-30">
                        <Select
                          value={String(filterState[col.key] ?? "")}
                          onChange={(e) => {
                            const selectedValue = col.filterValues?.find(
                              (val) => String(val) === e.target.value,
                            )
                            handleFilter(col.key, selectedValue as T[typeof col.key])
                          }}
                        >
                          <option value="">All</option>
                          {col.filterValues.map((value, idx) => (
                            <option key={idx} value={String(value)}>
                              {String(value)}
                            </option>
                          ))}
                        </Select>
                      </div>
                    )}
                  </div>
                </TableHead>
              ))}
            </TableRow>
          </TableHeader>
          <TableBody>
            {displayData.map(({ node }, idx) => (
              <TableRow
                key={idx}
                onClick={() => onClick?.(node)}
                className={onClick ? "cursor-pointer" : ""}
              >
                {columns.map((col) => (
                  <TableCell key={col.key as string}>
                    {col.render ? col.render(node[col.key], node) : String(node[col.key])}
                  </TableCell>
                ))}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      <div className="flex items-center justify-center space-x-2 py-4">
        <Button
          variant="outline"
          size="sm"
          onClick={handlePreviousPage}
          disabled={currentPage === 1}
        >
          <HiChevronLeft className="h-4 w-4" />
        </Button>
        <div className="flex items-center gap-1">
          <span className="text-sm font-medium">Page {currentPage}</span>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={handleNextPage}
          disabled={displayData.length < pageSize && !data.pageInfo.hasNextPage}
        >
          <HiChevronRight className="h-4 w-4" />
        </Button>
      </div>
    </>
  )
}

export default PaginatedTable

export const DEFAULT_PAGESIZE = 10
