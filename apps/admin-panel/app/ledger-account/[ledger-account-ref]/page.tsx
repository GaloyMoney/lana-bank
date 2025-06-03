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

import { useEffect, useState, use } from "react"

import { useRouter } from "next/navigation"
import { Button } from "@lana/web/ui/button"
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@lana/web/ui/collapsible"
import { FileDown } from "lucide-react"
import { IoCaretDownSharp, IoCaretForwardSharp } from "react-icons/io5"

import Link from "next/link"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { ExportCsvDialog } from "./export"

import { isUUID } from "@/lib/utils"
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
    children {
      id
      name
      code
    }
    balanceRange {
      __typename
      ... on UsdLedgerAccountBalanceRange {
        close {
          usdSettled: settled {
            debit
            credit
            net
          }
        }
      }
      ... on BtcLedgerAccountBalanceRange {
        close {
          btcSettled: settled {
            debit
            credit
            net
          }
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
          ledgerAccount {
            code
            closestAccountWithCode {
              code
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
  params: Promise<{
    "ledger-account-ref": string
  }>
}

const LedgerAccountPage: React.FC<LedgerAccountPageProps> = ({ params }) => {
  const router = useRouter()
  const t = useTranslations("ChartOfAccountsLedgerAccount")
  const [isExportDialogOpen, setIsExportDialogOpen] = useState(false)
  const { "ledger-account-ref": ref } = use(params)
  const isRefUUID = isUUID(ref)

  const [isAncestorsOpen, setIsAncestorsOpen] = useState(false)
  const [isChildrenOpen, setIsChildrenOpen] = useState(false)

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
      render: (recordedAt: string) => <DateWithTooltip value={recordedAt} />,
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
    {
      key: "ledgerAccount",
      label: t("table.columns.closestAccountWithCode"),
      render: (_, record) => {
        const closestAccountWithCode = record.ledgerAccount.closestAccountWithCode?.code
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
  ]

  const handleOpenExportDialog = () => {
    setIsExportDialogOpen(true)
  }

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
                <>
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
                            amount={ledgerAccount?.balanceRange?.close?.usdSettled.net}
                          />
                        ) : ledgerAccount?.balanceRange.__typename ===
                          "BtcLedgerAccountBalanceRange" ? (
                          <Balance
                            currency="btc"
                            amount={ledgerAccount?.balanceRange?.close?.btcSettled.net}
                          />
                        ) : (
                          "-"
                        )
                      }
                    />
                  </DetailsGroup>

                  <div className="flex flex-col space-y-2">
                    {ledgerAccount?.ancestors && ledgerAccount?.ancestors.length > 0 && (
                      <Collapsible
                        open={isAncestorsOpen}
                        onOpenChange={setIsAncestorsOpen}
                      >
                        <CollapsibleTrigger className="flex items-center space-x-1 font-semibold">
                          {isAncestorsOpen ? (
                            <IoCaretDownSharp />
                          ) : (
                            <IoCaretForwardSharp />
                          )}
                          <span>
                            {t("details.ancestors", {
                              n: ledgerAccount?.ancestors.length,
                            })}
                          </span>
                        </CollapsibleTrigger>
                        <CollapsibleContent className="max-w-[864px] pt-2">
                          <DataTable
                            onRowClick={(ancestor) =>
                              router.push(
                                `/ledger-account/${ancestor.code || ancestor.id}`,
                              )
                            }
                            cellClassName="!py-0 !h-10"
                            data={ledgerAccount?.ancestors || []}
                            columns={[
                              {
                                key: "code",
                                header: t("details.code"),
                                render: (code) => (
                                  <span className="font-mono text-xs font-bold">
                                    {code}
                                  </span>
                                ),
                              },
                              { key: "name", header: t("details.name") },
                            ]}
                            loading={loading}
                          />
                        </CollapsibleContent>
                      </Collapsible>
                    )}

                    {ledgerAccount?.children && ledgerAccount?.children.length > 0 && (
                      <Collapsible open={isChildrenOpen} onOpenChange={setIsChildrenOpen}>
                        <CollapsibleTrigger className="flex items-center space-x-1 font-semibold">
                          {isChildrenOpen ? (
                            <IoCaretDownSharp />
                          ) : (
                            <IoCaretForwardSharp />
                          )}
                          <span>
                            {t("details.children", { n: ledgerAccount?.children.length })}
                          </span>
                        </CollapsibleTrigger>
                        <CollapsibleContent className="max-w-[864px] pt-2">
                          <DataTable
                            onRowClick={({ code, id }) =>
                              router.push(`/ledger-account/${code || id}`)
                            }
                            cellClassName="!py-0 !h-10"
                            data={ledgerAccount?.children || []}
                            columns={[
                              {
                                key: "code",
                                header: t("details.code"),
                                render: (code) => (
                                  <span className="font-mono text-xs font-bold">
                                    {code}
                                  </span>
                                ),
                              },
                              { key: "name", header: t("details.name") },
                            ]}
                            loading={loading}
                          />
                        </CollapsibleContent>
                      </Collapsible>
                    )}
                  </div>
                </>
              )}
            </>
          )}
        </CardContent>
      </Card>
      <Card className="mt-2">
        <CardHeader>
          <CardTitle>
            <div className="flex items-center justify-between">
              {t("entriesTitle")}
              {ledgerAccount?.history?.edges &&
                ledgerAccount.history.edges.length > 0 && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleOpenExportDialog}
                    disabled={!ledgerAccount}
                  >
                    <FileDown className="h-4 w-4 mr-2" />
                    {t("exportCsv.buttons.export")}
                  </Button>
                )}
            </div>
          </CardTitle>
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

      {ledgerAccount && (
        <ExportCsvDialog
          isOpen={isExportDialogOpen}
          onClose={() => setIsExportDialogOpen(false)}
          ledgerAccountId={ledgerAccount.id}
        />
      )}
    </>
  )
}

export default LedgerAccountPage
