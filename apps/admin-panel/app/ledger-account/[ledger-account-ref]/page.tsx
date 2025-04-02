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

import { formatDate, isUUID } from "@/lib/utils"
import {
  useLedgerAccountByCodeQuery,
  LedgerAccountHistoryEntry,
  useLedgerAccountQuery,
} from "@/lib/graphql/generated"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import { DetailsGroup } from "@/components/details"
import Balance from "@/components/balance/balance"
import { useEffect } from "react"
import { useRouter } from "next/navigation"

gql`
  fragment LedgerAccountDetails on LedgerAccount {
    id
    name
    code
    balance {
      __typename
      ... on UsdLedgerAccountBalance {
        usdSettledBalance: settled
      }
      ... on BtcLedgerAccountBalance {
        btcSettledBalance: settled
      }
    }
    history(first: $first, after: $after) {
      edges {
        cursor
        node {
          __typename
          ... on BtcLedgerAccountHistoryEntry {
            txId
            recordedAt
            btcAmount {
              settled {
                debit
                credit
              }
            }
          }
          ... on UsdLedgerAccountHistoryEntry {
            txId
            recordedAt
            usdAmount {
              settled {
                debit
                credit
              }
            }
          }
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
    if (isRefUUID && ledgerAccount) {
      router.push(`/ledger-account/${ledgerAccount.code}`)
    }
  }, [ledgerAccount, isRefUUID])

  const columns: Column<LedgerAccountHistoryEntry>[] = [
    {
      key: "recordedAt",
      label: t("table.columns.recordedAt"),
      render: (recordedAt: string) => formatDate(recordedAt),
    },
    {
      key: "__typename",
      label: t("table.columns.currency"),
      render: (type: string | undefined) => (
        <div>{type === "UsdLedgerAccountHistoryEntry" ? "USD" : "BTC"}</div>
      ),
    },
    {
      key: "__typename",
      label: t("table.columns.debit"),
      render: (_?: string, record?: LedgerAccountHistoryEntry) => {
        if (!record) return null
        if (record.__typename === "UsdLedgerAccountHistoryEntry") {
          return <Balance amount={record?.usdAmount?.settled?.debit} currency="usd" />
        } else if (record.__typename === "BtcLedgerAccountHistoryEntry") {
          return <Balance amount={record?.btcAmount?.settled?.debit} currency="btc" />
        }
      },
    },
    {
      key: "__typename",
      label: t("table.columns.credit"),
      render: (_?: string, record?: LedgerAccountHistoryEntry) => {
        if (!record) return null
        if (record.__typename === "UsdLedgerAccountHistoryEntry") {
          return <Balance amount={record?.usdAmount?.settled?.credit} currency="usd" />
        } else if (record.__typename === "BtcLedgerAccountHistoryEntry") {
          return <Balance amount={record?.btcAmount?.settled?.credit} currency="btc" />
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
            {t("description", { code: ledgerAccount?.code })}
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
                    value={ledgerAccount?.code.replace(/\./g, "")}
                  />
                  <DetailItem
                    label={
                      ledgerAccount?.balance.__typename === "BtcLedgerAccountBalance"
                        ? t("details.btcBalance")
                        : t("details.usdBalance")
                    }
                    value={
                      ledgerAccount?.balance.__typename === "UsdLedgerAccountBalance" ? (
                        <Balance
                          currency="usd"
                          amount={ledgerAccount?.balance?.usdSettledBalance}
                        />
                      ) : ledgerAccount?.balance.__typename ===
                        "BtcLedgerAccountBalance" ? (
                        <Balance
                          currency="btc"
                          amount={ledgerAccount?.balance?.btcSettledBalance}
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
        </CardContent>
      </Card>
      <Card className="mt-2">
        <CardHeader>
          <CardTitle>{t("transactionsTitle")}</CardTitle>
        </CardHeader>
        <CardContent>
          <PaginatedTable<LedgerAccountHistoryEntry>
            columns={columns}
            data={ledgerAccount?.history as PaginatedData<LedgerAccountHistoryEntry>}
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
