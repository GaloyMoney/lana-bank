"use client"

import { use } from "react"
import { useTranslations } from "next-intl"

import { gql } from "@apollo/client"

import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import {
  LedgerTransaction,
  useLedgerTransactionsForTemplateCodeQuery,
} from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import DateWithTooltip from "@lana/web/components/date-with-tooltip"

gql`
  query LedgerTransactionsForTemplateCode(
    $templateCode: String!
    $first: Int!
    $after: String
  ) {
    ledgerTransactionsForTemplateCode(
      templateCode: $templateCode
      first: $first
      after: $after
    ) {
      edges {
        cursor
        node {
          id
          ledgerTransactionId
          createdAt
          description
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

type LedgerTransactionsForTemplateCodeProps = {
  params: Promise<{
    "transaction-template-code": string
  }>
}

const LedgerTransactionsForTemplateCode: React.FC<
  LedgerTransactionsForTemplateCodeProps
> = ({ params }) => {
  const { "transaction-template-code": transactionTemplateCode } = use(params)
  const t = useTranslations("LedgerTransactionsForTemplateCode")

  const { data, loading, error, fetchMore } = useLedgerTransactionsForTemplateCodeQuery({
    variables: {
      templateCode: transactionTemplateCode,
      first: DEFAULT_PAGESIZE,
    },
  })

  const columns: Column<LedgerTransaction>[] = [
    {
      key: "createdAt",
      label: t("table.headers.createdAt"),
      render: (date) => <DateWithTooltip value={date} />,
    },
    {
      key: "description",
      label: t("table.headers.description"),
    },
  ]

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title", { code: `"${transactionTemplateCode}"` })}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        {error ? (
          <p className="text-destructive text-sm">{t("errors.general")}</p>
        ) : (
          <PaginatedTable<LedgerTransaction>
            columns={columns}
            pageSize={DEFAULT_PAGESIZE}
            data={
              data?.ledgerTransactionsForTemplateCode as PaginatedData<LedgerTransaction>
            }
            loading={loading}
            fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
            navigateTo={({ ledgerTransactionId }) =>
              `/ledger-transaction/${ledgerTransactionId}`
            }
            noDataText={t("table.noData")}
          />
        )}
      </CardContent>
    </Card>
  )
}

export default LedgerTransactionsForTemplateCode
