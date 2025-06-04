"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { use } from "react"

import { DetailItem } from "@lana/web/components/details"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import Link from "next/link"

import { formatDate } from "@lana/web/utils"

import { useLedgerTransactionQuery, DebitOrCredit } from "@/lib/graphql/generated"
import { DetailsGroup } from "@/components/details"
import Balance from "@/components/balance/balance"
import DataTable from "@/components/data-table"

gql`
  query LedgerTransaction($id: UUID!) {
    ledgerTransaction(id: $id) {
      id
      ledgerTransactionId
      createdAt
      description
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
      <Card>
        <CardHeader>
          <CardTitle>{t("title")}</CardTitle>
          <CardDescription>{t("description")}</CardDescription>
        </CardHeader>
        <CardContent>
          {error ? (
            <p className="text-destructive text-sm">{error?.message}</p>
          ) : (
            <>
              {!loading && (
                <DetailsGroup className="mb-4">
                  <DetailItem
                    label={t("details.description")}
                    value={data?.ledgerTransaction?.description}
                  />
                  <DetailItem
                    label={t("details.createdAt")}
                    value={formatDate(data?.ledgerTransaction?.createdAt)}
                  />
                </DetailsGroup>
              )}
            </>
          )}
        </CardContent>
      </Card>
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
