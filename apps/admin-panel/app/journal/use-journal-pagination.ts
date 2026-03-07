import { useState, useMemo, useCallback } from "react"

import {
  useJournalEntriesQuery,
  JournalEntriesSort,
  JournalEntriesSortBy,
  SortDirection,
} from "@/lib/graphql/generated"

const PAGE_SIZE = 50

const DEFAULT_SORT: JournalEntriesSort = {
  by: JournalEntriesSortBy.CreatedAt,
  direction: SortDirection.Desc,
}

export const useJournalPagination = () => {
  const [currentPage, setCurrentPage] = useState(1)
  const [sort, setSort] = useState<JournalEntriesSort>(DEFAULT_SORT)

  const { data, loading, error, fetchMore } = useJournalEntriesQuery({
    variables: { first: PAGE_SIZE, sort },
  })

  const displayData = useMemo(() => {
    if (!data?.journalEntries?.edges) return []
    const startIdx = (currentPage - 1) * PAGE_SIZE
    const endIdx = startIdx + PAGE_SIZE
    return data.journalEntries.edges.slice(startIdx, endIdx).map((edge) => edge.node)
  }, [data, currentPage])

  const handleNextPage = useCallback(async () => {
    const nextPage = currentPage + 1
    const requiredDataLength = nextPage * PAGE_SIZE
    const currentDataLength = data?.journalEntries?.edges.length || 0

    if (
      currentDataLength < requiredDataLength &&
      data?.journalEntries?.pageInfo.hasNextPage
    ) {
      await fetchMore({ variables: { after: data.journalEntries.pageInfo.endCursor } })
    }
    setCurrentPage(nextPage)
  }, [data, currentPage, fetchMore])

  const handlePreviousPage = useCallback(() => {
    setCurrentPage((prev) => (prev > 1 ? prev - 1 : prev))
  }, [])

  const onSort = useCallback(
    (newSort: JournalEntriesSort) => {
      setSort(newSort)
      setCurrentPage(1)
    },
    [],
  )

  return {
    loading,
    error,
    displayData,
    currentPage,
    hasNextPage: data?.journalEntries?.pageInfo.hasNextPage,
    handleNextPage,
    handlePreviousPage,
    pageSize: PAGE_SIZE,
    sort,
    onSort,
  }
}
