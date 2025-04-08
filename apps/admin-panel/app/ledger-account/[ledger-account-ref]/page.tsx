"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { DetailItem } from "@lana/web/components/details"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import { useEffect } from "react"

import { useRouter } from "next/navigation"

import { formatDate, isUUID } from "@/lib/utils"
import {
  useLedgerAccountByCodeQuery,
  useLedgerAccountQuery,
  JournalEntry,
  DebitOrCredit,
} from "@/lib/graphql/generated"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import { DetailsGroup } from "@/components/details"
import Balance from "@/components/balance/balance"
import DataTable from "@/components/data-table"

gql`
  fragment LedgerAccountDetails on LedgerAccount {
    id
    name
    code
    ancestors {
      id
      name
      code
    }
    balanceRange {
      __typename
      ... on UsdLedgerAccountBalanceRange {
        start {
          usdSettled: settled
          usdPending: pending
          usdEncumbrance: encumbrance
        }
        diff {
          usdSettledDiff: settled
          usdPendingDiff: pending
          usdEncumbranceDiff: encumbrance
        }
        end {
          usdSettledEnd: settled
          usdPendingEnd: pending
          usdEncumbranceEnd: encumbrance
        }
      }
      ... on BtcLedgerAccountBalanceRange {
        start {
          btcSettled: settled
          btcPending: pending
          btcEncumbrance: encumbrance
        }
        diff {
          btcSettledDiff: settled
          btcPendingDiff: pending
          btcEncumbranceDiff: encumbrance
        }
        end {
          btcSettledEnd: settled
          btcPendingEnd: pending
          btcEncumbranceEnd: encumbrance
        }
      }
    }
    history(first: $first, after: $after) {
      edges {
        cursor
        node {
          id
          entryId
          txId
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
          description
          direction
          layer
          createdAt
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

  query LedgerAccountByCode($code: String!, $first: Int!, $after: String) {
    ledgerAccountByCode(code: $code) {
      ...LedgerAccountDetails
    }
  }

  query LedgerAccount($id: UUID!, $first: Int!, $after: String) {
    ledgerAccount(id: $id) {
      ...LedgerAccountDetails
    }
  }
`

type LedgerAccountPageProps = {
  params: {
    "ledger-account-ref": string
  }
}

const LedgerAccountPage: React.FC<LedgerAccountPageProps> = ({ params }) => {
  const router = useRouter()
  const t = useTranslations("ChartOfAccountsLedgerAccount")
  const { "ledger-account-ref": ref } = params
  const isRefUUID = isUUID(ref)

  const ledgerAccountByCodeData = useLedgerAccountByCodeQuery({
    variables: { code: ref, first: DEFAULT_PAGESIZE },
    skip: isRefUUID,
  })
  const ledgerAccountData = useLedgerAccountQuery({
    variables: { id: ref, first: DEFAULT_PAGESIZE },
    skip: !isRefUUID,
  })

  const ledgerAccount = isRefUUID
    ? ledgerAccountData.data?.ledgerAccount
    : ledgerAccountByCodeData.data?.ledgerAccountByCode

  const { loading, error, fetchMore } = isRefUUID
    ? ledgerAccountData
    : ledgerAccountByCodeData

  useEffect(() => {
    if (isRefUUID && ledgerAccount && ledgerAccount.code) {
      router.push(`/ledger-account/${ledgerAccount.code}`)
    }
  }, [ledgerAccount, isRefUUID, router])

  const columns: Column<JournalEntry>[] = [
    {
      key: "createdAt",
      label: t("table.columns.recordedAt"),
      render: (recordedAt: string) => formatDate(recordedAt),
    },
    {
      key: "amount",
      label: t("table.columns.currency"),
      render: (amount) => <div>{amount.__typename === "UsdAmount" ? "USD" : "BTC"}</div>,
    },
    {
      key: "__typename",
      label: t("table.columns.debit"),
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
      key: "__typename",
      label: t("table.columns.credit"),
      render: (_, record) => {
        if (record.direction !== DebitOrCredit.Credit) return null
        if (record.amount.__typename === "UsdAmount") {
          return <Balance amount={record?.amount.usd} currency="usd" />
        } else if (record.amount.__typename === "BtcAmount") {
          return <Balance amount={record?.amount.btc} currency="btc" />
        }
      },
    },
  ]

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>{t("title")}</CardTitle>
          <CardDescription>
            {ledgerAccount?.code
              ? t("descriptionWithCode", { code: ledgerAccount?.code })
              : t("description")}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {error ? (
            <p className="text-destructive text-sm">{error?.message}</p>
          ) : (
            <>
              {!loading && (
                <DetailsGroup columns={3} className="mb-4">
                  <DetailItem label={t("details.name")} value={ledgerAccount?.name} />
                  <DetailItem
                    label={t("details.code")}
                    value={ledgerAccount?.code || "-"}
                  />
                  <DetailItem
                    label={
                      ledgerAccount?.balanceRange.__typename ===
                      "BtcLedgerAccountBalanceRange"
                        ? t("details.btcBalance")
                        : t("details.usdBalance")
                    }
                    value={
                      ledgerAccount?.balanceRange.__typename ===
                      "UsdLedgerAccountBalanceRange" ? (
                        <Balance
                          currency="usd"
                          amount={ledgerAccount?.balanceRange?.diff?.usdSettledDiff}
                        />
                      ) : ledgerAccount?.balanceRange.__typename ===
                        "BtcLedgerAccountBalanceRange" ? (
                        <Balance
                          currency="btc"
                          amount={ledgerAccount?.balanceRange?.diff?.btcSettledDiff}
                        />
                      ) : (
                        <>N/A</>
                      )
                    }
                  />
                </DetailsGroup>
              )}
            </>
          )}
          {ledgerAccount?.ancestors.length !== 0 && (
            <div>
              <DetailItem
                label={t("details.ancestors")}
                value={
                  <DataTable
                    autoFocus={false}
                    data={ledgerAccount?.ancestors || []}
                    columns={[
                      { key: "name", header: t("details.name") },
                      { key: "code", header: t("details.code") },
                    ]}
                    loading={loading}
                    emptyMessage={t("details.noAncestors")}
                    navigateTo={(ancestor) => `/ledger-account/${ancestor.code}`}
                  />
                }
              />
            </div>
          )}
        </CardContent>
      </Card>
      <Card className="mt-2">
        <CardHeader>
          <CardTitle>{t("transactionsTitle")}</CardTitle>
        </CardHeader>
        <CardContent>
          <PaginatedTable<JournalEntry>
            columns={columns}
            data={ledgerAccount?.history as PaginatedData<JournalEntry>}
            pageSize={DEFAULT_PAGESIZE}
            fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
            loading={loading}
            noDataText={t("table.noData")}
          />
        </CardContent>
      </Card>
    </>
  )
}

export default LedgerAccountPage
