"use client"

import { useState, useEffect } from "react"
import {
  HiChevronUp,
  HiChevronDown,
  HiSelector,
  HiChevronLeft,
  HiChevronRight,
} from "react-icons/hi"

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
}

const PaginatedTable = <T,>({
  columns,
  data,
  pageSize,
  fetchMore,
  onSort,
  onFilter,
}: PaginatedTableProps<T>): React.ReactElement => {
  const [sortState, setSortState] = useState<{
    column: keyof T | null
    direction: "ASC" | "DESC" | null
  }>({ column: null, direction: null })

  const [filterState, setFilterState] = useState<Partial<Record<keyof T, T[keyof T]>>>({})

  const [currentPage, setCurrentPage] = useState(1)
  const [displayData, setDisplayData] = useState<{ node: T }[]>([])

  useEffect(() => {
    // Update displayData when data or currentPage changes
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
    const maxDataRequired = currentPage * pageSize + pageSize // Data needed for next page

    if (totalDataLoaded < maxDataRequired && data.pageInfo.hasNextPage) {
      // Need to fetch more data
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
    <div className="overflow-auto h-full w-full">
      <table className="w-full min-w-max table-auto text-left">
        <thead>
          <tr>
            {columns.map((col) => (
              <th
                key={col.key as string}
                className="pt-4 pb-2 text-heading text-title-sm"
              >
                <div className="flex items-center">
                  <span className="text-title-sm">{col.label}</span>
                  {col.sortable && (
                    <button
                      onClick={() => handleSort(col.key)}
                      className="ml-2 text-gray-500 hover:text-gray-700 focus:outline-none"
                    >
                      {sortState.column === col.key ? (
                        sortState.direction === "ASC" ? (
                          <HiChevronUp className="w-4 h-4" />
                        ) : (
                          <HiChevronDown className="w-4 h-4" />
                        )
                      ) : (
                        <HiSelector className="w-4 h-4" />
                      )}
                    </button>
                  )}
                  {col.filterValues && (
                    <select
                      value={String(filterState[col.key] ?? "")}
                      onChange={(e) => {
                        const value = col.filterValues?.find(
                          (val) => String(val) === e.target.value,
                        )
                        handleFilter(col.key, value as T[typeof col.key])
                      }}
                      className="ml-2 border border-gray-300 rounded text-sm"
                    >
                      <option value="">All</option>
                      {col.filterValues.map((value, idx) => (
                        <option key={idx} value={String(value)}>
                          {String(value)}
                        </option>
                      ))}
                    </select>
                  )}
                </div>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {displayData.map(({ node }, idx) => (
            <tr key={idx} className="hover:bg-gray-100">
              {columns.map((col) => (
                <td key={col.key as string} className="text-body-md p-1 text-body-sm">
                  {col.render ? col.render(node[col.key], node) : String(node[col.key])}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>

      {/* Pagination controls */}
      <div className="flex justify-center mt-4">
        <nav className="inline-flex -space-x-px">
          <button
            onClick={handlePreviousPage}
            disabled={currentPage === 1}
            className={`px-3 py-1 border border-gray-300 rounded-l-md hover:bg-gray-100 ${
              currentPage === 1 ? "opacity-50 cursor-not-allowed" : ""
            }`}
          >
            <HiChevronLeft className="w-5 h-5" />
          </button>
          <span className="px-3 py-1 border-t border-b border-gray-300">
            Page {currentPage}
          </span>
          <button
            onClick={handleNextPage}
            disabled={displayData.length < pageSize && !data.pageInfo.hasNextPage}
            className={`px-3 py-1 border border-gray-300 rounded-r-md hover:bg-gray-100 ${
              displayData.length < pageSize && !data.pageInfo.hasNextPage
                ? "opacity-50 cursor-not-allowed"
                : ""
            }`}
          >
            <HiChevronRight className="w-5 h-5" />
          </button>
        </nav>
      </div>
    </div>
  )
}

export default PaginatedTable
