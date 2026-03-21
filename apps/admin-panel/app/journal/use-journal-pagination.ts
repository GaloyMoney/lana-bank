import { useState, useMemo, useCallback } from "react"

import { useLedgerEntriesQuery } from "@/lib/graphql/generated"

const PAGE_SIZE = 50

export const useJournalPagination = () => {
  const [currentPage, setCurrentPage] = useState(1)

  const { data, loading, error, fetchMore } = useLedgerEntriesQuery({
    variables: { first: PAGE_SIZE },
  })

  const displayData = useMemo(() => {
    if (!data?.ledgerEntries?.edges) return []
    const startIdx = (currentPage - 1) * PAGE_SIZE
    const endIdx = startIdx + PAGE_SIZE
    return data.ledgerEntries.edges.slice(startIdx, endIdx).map((edge) => edge.node)
  }, [data, currentPage])

  const handleNextPage = useCallback(async () => {
    const nextPage = currentPage + 1
    const requiredDataLength = nextPage * PAGE_SIZE
    const currentDataLength = data?.ledgerEntries?.edges.length || 0

    if (
      currentDataLength < requiredDataLength &&
      data?.ledgerEntries?.pageInfo.hasNextPage
    ) {
      await fetchMore({ variables: { after: data.ledgerEntries.pageInfo.endCursor } })
    }
    setCurrentPage(nextPage)
  }, [data, currentPage, fetchMore])

  const handlePreviousPage = useCallback(() => {
    setCurrentPage((prev) => (prev > 1 ? prev - 1 : prev))
  }, [])

  return {
    loading,
    error,
    displayData,
    currentPage,
    hasNextPage:
      data?.ledgerEntries?.pageInfo.hasNextPage ||
      (data?.ledgerEntries?.edges.length || 0) > currentPage * PAGE_SIZE,
    handleNextPage,
    handlePreviousPage,
    pageSize: PAGE_SIZE,
  }
}
