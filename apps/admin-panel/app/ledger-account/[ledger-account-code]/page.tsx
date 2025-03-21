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

import { formatDate } from "@/lib/utils"
import {
  useLedgerAccountByCodeQuery,
  LedgerAccountHistoryEntry,
} from "@/lib/graphql/generated"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import { DetailsGroup } from "@/components/details"
import Balance from "@/components/balance/balance"

gql`
  query LedgerAccountByCode($code: String!, $first: Int!, $after: String) {
    ledgerAccountByCode(code: $code) {
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
  }
`

type LedgerAccountPageProps = {
  params: {
    "ledger-account-code": string
  }
}

const LedgerAccountPage: React.FC<LedgerAccountPageProps> = ({ params }) => {
  const t = useTranslations("ChartOfAccountsLedgerAccount")
  const { "ledger-account-code": code } = params

  const { data, loading, error, fetchMore } = useLedgerAccountByCodeQuery({
    variables: {
      code,
      first: DEFAULT_PAGESIZE,
    },
  })

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
          return <div>{record?.usdAmount?.settled?.debit}</div>
        } else if (record.__typename === "BtcLedgerAccountHistoryEntry") {
          return <div>{record?.btcAmount?.settled?.debit}</div>
        }
      },
    },
    {
      key: "__typename",
      label: t("table.columns.credit"),
      render: (_?: string, record?: LedgerAccountHistoryEntry) => {
        if (!record) return null
        if (record.__typename === "UsdLedgerAccountHistoryEntry") {
          return <div>{record?.usdAmount?.settled?.credit}</div>
        } else if (record.__typename === "BtcLedgerAccountHistoryEntry") {
          return <div>{record?.btcAmount?.settled?.credit}</div>
        }
      },
    },
  ]

  return (
    <>
      <Card className="mb-10">
        <CardHeader>
          <CardTitle>{t("title")}</CardTitle>
          <CardDescription>
            {t("description", { code: data?.ledgerAccountByCode?.code })}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {error ? (
            <p className="text-destructive text-sm">{error?.message}</p>
          ) : (
            <>
              {!loading && (
                <DetailsGroup columns={3} className="mb-4">
                  <DetailItem
                    label={t("details.name")}
                    value={data?.ledgerAccountByCode?.name}
                  />
                  <DetailItem
                    label={t("details.code")}
                    value={data?.ledgerAccountByCode?.code.replace(/\./g, "")}
                  />
                  <DetailItem
                    label={
                      data?.ledgerAccountByCode?.balance.__typename ===
                      "BtcLedgerAccountBalance"
                        ? t("details.btcBalance")
                        : t("details.usdBalance")
                    }
                    value={
                      data?.ledgerAccountByCode?.balance.__typename ===
                      "UsdLedgerAccountBalance" ? (
                        <Balance
                          currency="usd"
                          amount={data?.ledgerAccountByCode?.balance?.usdSettledBalance}
                        />
                      ) : data?.ledgerAccountByCode?.balance.__typename ===
                        "BtcLedgerAccountBalance" ? (
                        <Balance
                          currency="btc"
                          amount={data?.ledgerAccountByCode?.balance?.btcSettledBalance}
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
      <Card className="-mt-5">
        <CardHeader>
          <CardTitle>{t("transactionsTitle")}</CardTitle>
        </CardHeader>
        <CardContent>
          <PaginatedTable<LedgerAccountHistoryEntry>
            columns={columns}
            data={
              data?.ledgerAccountByCode
                ?.history as PaginatedData<LedgerAccountHistoryEntry>
            }
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
