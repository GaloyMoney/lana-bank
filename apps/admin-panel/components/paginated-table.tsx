/* eslint-disable @typescript-eslint/no-unused-expressions */

"use client"

import { useState } from "react"
import {
  HiChevronUp,
  HiChevronDown,
  HiSelector,
  HiChevronLeft,
  HiChevronRight,
} from "react-icons/hi"

export interface Column<T, K extends keyof T = keyof T> {
  key: K
  label: string
  sortable?: boolean
  filterValues?: Array<T[K]>
  render?: (value: T[K], record: T) => React.ReactNode
}

interface PageInfo {
  endCursor: string
  startCursor: string
  hasNextPage: boolean
  hasPreviousPage: boolean
}

export interface PaginatedData<T> {
  edges: { node: T }[]
  pageInfo: PageInfo
}

interface PaginatedTableProps<T> {
  columns: Column<T>[]
  data: PaginatedData<T>
  fetchMore: (cursor: string) => Promise<void>
  onSort?: (column: keyof T, sortDirection: "ASC" | "DESC") => void
  onFilter?: (column: keyof T, value: T[keyof T]) => void
}

const PaginatedTable = <T,>({
  columns,
  data,
  onSort,
  onFilter,
}: PaginatedTableProps<T>): React.ReactElement => {
  const [sortState, setSortState] = useState<{
    column: keyof T | null
    direction: "ASC" | "DESC" | null
  }>({ column: null, direction: null })

  const [filterState, setFilterState] = useState<Partial<Record<keyof T, T[keyof T]>>>({})

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

  const handlePreviousPage = () => {}
  const handleNextPage = () => {}

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
                  <span>{col.label}</span>
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
          {data.edges.map(({ node }, idx) => (
            <tr key={idx} className="hover:bg-gray-100">
              {columns.map((col) => (
                <td key={col.key as string} className="text-body-md p-1">
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
            disabled={!data.pageInfo.hasPreviousPage}
            className={`px-3 py-1 border border-gray-300 rounded-l-md hover:bg-gray-100 ${
              !data.pageInfo.hasPreviousPage ? "opacity-50 cursor-not-allowed" : ""
            }`}
          >
            <HiChevronLeft className="w-5 h-5" />
          </button>
          <button
            onClick={handleNextPage}
            disabled={!data.pageInfo.hasNextPage}
            className={`px-3 py-1 border border-gray-300 rounded-r-md hover:bg-gray-100 ${
              !data.pageInfo.hasNextPage ? "opacity-50 cursor-not-allowed" : ""
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
