"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { use } from "react"

import Link from "next/link"

import { formatDate } from "@lana/web/utils"

import { Card, CardContent, CardHeader, CardTitle } from "@lana/web/ui/card"

import { useLedgerTransactionQuery, DebitOrCredit } from "@/lib/graphql/generated"
import { DetailsCard } from "@/components/details"
import Balance from "@/components/balance/balance"
import DataTable from "@/components/data-table"

gql`
  query LedgerTransaction($id: UUID!) {
    ledgerTransaction(id: $id) {
      id
      ledgerTransactionId
      createdAt
      description
      effective
      entries {
        id
        entryId
        entryType
        amount {
          __typename
          ... on UsdAmount {
            usd
          }
          ... on BtcAmount {
            btc
          }
        }
        direction
        layer
        ledgerAccount {
          id
          code
          name
          closestAccountWithCode {
            code
          }
        }
      }
    }
  }
`

type LedgerTransactionPageProps = {
  params: Promise<{
    "ledger-transaction-id": string
  }>
}

const LedgerTransactionPage: React.FC<LedgerTransactionPageProps> = ({ params }) => {
  const t = useTranslations("LedgerTransaction")
  const { "ledger-transaction-id": id } = use(params)

  const { data, loading, error } = useLedgerTransactionQuery({
    variables: { id },
  })

  return (
    <>
      <DetailsCard
        title={t("title")}
        description={t("description")}
        columns={3}
        details={[
          {
            label: t("details.description"),
            value: data?.ledgerTransaction?.description,
          },
          {
            label: t("details.createdAt"),
            value: formatDate(data?.ledgerTransaction?.createdAt),
          },
          {
            label: t("details.effective"),
            value: formatDate(data?.ledgerTransaction?.effective, {
              includeTime: false,
            }),
          },
        ]}
        errorMessage={error?.message}
      />
      <Card className="mt-2">
        <CardHeader>
          <CardTitle>{t("entriesTitle")}</CardTitle>
        </CardHeader>
        <CardContent>
          <DataTable
            data={data?.ledgerTransaction?.entries || []}
            columns={[
              {
                key: "ledgerAccount",
                header: t("table.ledgerAccount"),
                render: (account) => {
                  const accountName = account.name || account.code
                  return (
                    <Link
                      href={`/ledger-account/${account.code || account.id}`}
                      className="hover:underline"
                    >
                      {accountName}
                    </Link>
                  )
                },
              },
              { key: "entryType", header: t("table.entryType") },
              {
                key: "direction",
                header: t("table.debit"),
                render: (_, record) => {
                  if (record.direction !== DebitOrCredit.Debit) return null
                  if (record.amount.__typename === "UsdAmount") {
                    return <Balance amount={record?.amount.usd} currency="usd" />
                  } else if (record.amount.__typename === "BtcAmount") {
                    return <Balance amount={record?.amount.btc} currency="btc" />
                  }
                },
              },
              {
                key: "direction",
                header: t("table.credit"),
                render: (_, record) => {
                  if (record.direction !== DebitOrCredit.Credit) return null
                  if (record.amount.__typename === "UsdAmount") {
                    return <Balance amount={record?.amount.usd} currency="usd" />
                  } else if (record.amount.__typename === "BtcAmount") {
                    return <Balance amount={record?.amount.btc} currency="btc" />
                  }
                },
              },
              {
                key: "ledgerAccount",
                header: t("table.closestAccountWithCode"),
                render: (_, record) => {
                  const closestAccountWithCode =
                    record.ledgerAccount?.closestAccountWithCode?.code
                  return (
                    <Link
                      href={`/ledger-account/${closestAccountWithCode}`}
                      className="hover:underline"
                    >
                      {closestAccountWithCode}
                    </Link>
                  )
                },
              },
            ]}
            loading={loading}
            emptyMessage={t("noEntries")}
          />
        </CardContent>
      </Card>
    </>
  )
}

export default LedgerTransactionPage
