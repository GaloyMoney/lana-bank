"use client"

import { useTranslations } from "next-intl"

import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@lana/web/ui/card"

import { gql } from "@apollo/client"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"

import {
  TransactionTemplate,
  useTransactionTemplatesQuery,
} from "@/lib/graphql/generated"

gql`
  query TransactionTemplates($first: Int!, $after: String) {
    transactionTemplates(first: $first, after: $after) {
      edges {
        cursor
        node {
          id
          transactionTemplateId
          code
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

const TransactionTemplates = () => {
  const t = useTranslations("TransactionTemplates")
  const tCode = useTranslations("TemplateCodes")

  const { data, loading, error, fetchMore } = useTransactionTemplatesQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
  })

  const columns: Column<TransactionTemplate>[] = [
    {
      key: "code",
      label: t("code"),
      render: (code) => (tCode.has(code) ? tCode(code) : code),
    },
  ]

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        {error ? (
          <p className="text-destructive text-sm">{t("errors.general")}</p>
        ) : (
          <PaginatedTable<TransactionTemplate>
            columns={columns}
            pageSize={DEFAULT_PAGESIZE}
            data={data?.transactionTemplates as PaginatedData<TransactionTemplate>}
            loading={loading}
            fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
            navigateTo={({ code }) => `/transaction-templates/${code}`}
            noDataText={t("table.noData")}
          />
        )}
      </CardContent>
    </Card>
  )
}

export default TransactionTemplates
