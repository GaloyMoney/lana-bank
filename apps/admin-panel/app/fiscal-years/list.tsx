"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { useState, useEffect } from "react"
import { Plus } from "lucide-react"

import { Button } from "@lana/web/ui/button"
import { Card, CardContent } from "@lana/web/ui/card"
import { formatUTCDateOnly } from "@lana/web/utils"

import { useCreateContext } from "../create"

import { FiscalYearStatusBadge } from "./status-badge"
import { InitFiscalYearDialog } from "./init-fiscal-year"

import { FiscalYear, useFiscalYearsQuery } from "@/lib/graphql/generated"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"

gql`
  fragment FiscalYearFields on FiscalYear {
    id
    fiscalYearId
    chartId
    openedAsOf
    isOpen
    reference
    isLastMonthOfYearClosed
  }

  query FiscalYears($first: Int!, $after: String) {
    fiscalYears(first: $first, after: $after) {
      edges {
        cursor
        node {
          ...FiscalYearFields
        }
      }
      pageInfo {
        endCursor
        startCursor
        hasNextPage
        hasPreviousPage
      }
    }
  }
`

const FiscalYearsList = () => {
  const t = useTranslations("FiscalYears.table")
  const tInit = useTranslations("FiscalYears.init")
  const [openInitDialog, setOpenInitDialog] = useState(false)

  const { data, loading, error, fetchMore } = useFiscalYearsQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
  })

  const { setLatestFiscalYear } = useCreateContext()

  useEffect(() => {
    const latestFiscalYear = data?.fiscalYears?.edges?.[0]?.node ?? null
    setLatestFiscalYear(latestFiscalYear)
    return () => setLatestFiscalYear(null)
  }, [data, setLatestFiscalYear])

  const columns: Column<FiscalYear>[] = [
    {
      key: "openedAsOf",
      label: t("headers.openedAsOf"),
      render: (openedAsOf) => formatUTCDateOnly(openedAsOf) ?? "-",
    },
    {
      key: "isOpen",
      label: t("headers.status"),
      render: (isOpen) => <FiscalYearStatusBadge isOpen={isOpen} />,
    },
  ]

  const hasFiscalYears = data?.fiscalYears?.edges && data.fiscalYears.edges.length > 0

  if (!loading && !hasFiscalYears) {
    return (
      <>
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <p className="text-muted-foreground text-sm mb-4 text-center max-w-md">
              {tInit("description")}
            </p>
            <Button onClick={() => setOpenInitDialog(true)}>
              <Plus className="h-4 w-4" />
              {t("initFiscalYear")}
            </Button>
          </CardContent>
        </Card>
        <InitFiscalYearDialog open={openInitDialog} onOpenChange={setOpenInitDialog} />
      </>
    )
  }

  return (
    <>
      {error && <p className="text-destructive text-sm">{error?.message}</p>}
      <PaginatedTable<FiscalYear>
        columns={columns}
        data={data?.fiscalYears as PaginatedData<FiscalYear>}
        loading={loading}
        fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
        pageSize={DEFAULT_PAGESIZE}
        navigateTo={(fiscalYear) => `/fiscal-years/${fiscalYear.reference}`}
      />
    </>
  )
}

export default FiscalYearsList
