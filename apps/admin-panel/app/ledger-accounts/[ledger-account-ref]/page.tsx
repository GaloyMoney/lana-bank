"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { Card, CardContent, CardHeader, CardTitle } from "@lana/web/ui/card"

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

import { validate } from "uuid"

import { ExportCsvDialog } from "./export"

import {
  useLedgerAccountByCodeQuery,
  useLedgerAccountQuery,
  JournalEntry,
  DebitOrCredit,
  LedgerAccountDetailsFragment,
} from "@/lib/graphql/generated"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import { DetailsCard } from "@/components/details"
import Balance from "@/components/balance/balance"
import DataTable from "@/components/data-table"
import LayerLabel from "@/app/journal/layer-label"
import { useCreateContext } from "@/app/create"

gql`
  fragment LedgerAccountDetails on LedgerAccount {
    id
    ledgerAccountId
    name
    code
    ancestors {
      id
      ledgerAccountId
      name
      code
    }
    children {
      id
      ledgerAccountId
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
  const isRefUUID = validate(ref)
  const { setLedgerAccount } = useCreateContext()

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
      router.push(`/ledger-accounts/${ledgerAccount.code}`)
    }
  }, [ledgerAccount, isRefUUID, router])

  useEffect(() => {
    if (ledgerAccount) {
      setLedgerAccount(ledgerAccount)
    }
    return () => {
      setLedgerAccount(null)
    }
  }, [ledgerAccount, setLedgerAccount])

  const columns: Column<JournalEntry>[] = [
    {
      key: "createdAt",
      label: t("table.columns.recordedAt"),
      render: (recordedAt: string) => <DateWithTooltip value={recordedAt} />,
    },
    {
      key: "ledgerTransaction",
      label: t("table.columns.transactionId"),
      render: (_, record) => {
        return (
          <Link className="hover:underline" href={`/ledger-transactions/${record.txId}`}>
            {record.txId.substring(0, 6)}...
            {record.txId.substring(record.txId.length - 6)}
          </Link>
        )
      },
    },
    {
      key: "ledgerAccount",
      label: t("table.columns.closestAccountWithCode"),
      render: (_, record) => {
        const closestAccountWithCode = record.ledgerAccount.closestAccountWithCode?.code
        return (
          <Link
            href={`/ledger-accounts/${closestAccountWithCode}`}
            className="hover:underline"
          >
            {closestAccountWithCode}
          </Link>
        )
      },
    },
    {
      key: "layer",
      label: t("table.columns.layer"),
      render: (layer) => <LayerLabel value={layer} />,
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

  const handleOpenExportDialog = () => {
    setIsExportDialogOpen(true)
  }

  const details = [
    { label: t("details.name"), value: ledgerAccount?.name },
    { label: t("details.code"), value: ledgerAccount?.code || "-" },
    {
      label:
        ledgerAccount?.balanceRange.__typename === "BtcLedgerAccountBalanceRange"
          ? t("details.btcBalance")
          : t("details.usdBalance"),
      value:
        ledgerAccount?.balanceRange.__typename === "UsdLedgerAccountBalanceRange" ? (
          <Balance
            currency="usd"
            amount={ledgerAccount?.balanceRange?.close?.usdSettled.net}
          />
        ) : ledgerAccount?.balanceRange.__typename === "BtcLedgerAccountBalanceRange" ? (
          <Balance
            currency="btc"
            amount={ledgerAccount?.balanceRange?.close?.btcSettled.net}
          />
        ) : (
          "-"
        ),
    },
  ]

  return (
    <>
      <DetailsCard
        title={t("title")}
        description={
          ledgerAccount?.code
            ? t("descriptionWithCode", { code: ledgerAccount?.code })
            : t("description")
        }
        details={details}
        columns={3}
        errorMessage={error?.message}
      />

      {(() => {
        const ancestors = ledgerAccount?.ancestors || []
        const children = ledgerAccount?.children || []
        const hasRelatives = ancestors.length > 0 || children.length > 0

        if (loading || !hasRelatives) return null

        return (
          <Card className="mt-2">
            <CardContent>
              <div className="flex flex-col space-y-2">
                {ancestors.length > 0 && (
                  <CollapsibleAccountSection
                    title={t("details.ancestors", { n: ancestors.length })}
                    isOpen={isAncestorsOpen}
                    onOpenChange={setIsAncestorsOpen}
                    data={ancestors}
                    onRowClick={(ancestor) =>
                      router.push(
                        `/ledger-accounts/${ancestor.code || ancestor.ledgerAccountId}`,
                      )
                    }
                    t={t}
                    loading={loading}
                  />
                )}
                {children.length > 0 && (
                  <CollapsibleAccountSection
                    title={t("details.children", { n: children.length })}
                    isOpen={isChildrenOpen}
                    onOpenChange={setIsChildrenOpen}
                    data={children}
                    onRowClick={({ code, ledgerAccountId }) =>
                      router.push(`/ledger-accounts/${code || ledgerAccountId}`)
                    }
                    t={t}
                    loading={loading}
                  />
                )}
              </div>
            </CardContent>
          </Card>
        )
      })()}
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
          ledgerAccountId={ledgerAccount.ledgerAccountId}
        />
      )}
    </>
  )
}

export default LedgerAccountPage

type CollapsibleAccountSectionProps = {
  title: string
  isOpen: boolean
  onOpenChange: (open: boolean) => void
  data:
    | LedgerAccountDetailsFragment["ancestors"]
    | LedgerAccountDetailsFragment["children"]
  onRowClick: (item: LedgerAccountDetailsFragment["ancestors"][0]) => void
  t: (key: string) => string
  loading: boolean
}

const CollapsibleAccountSection: React.FC<CollapsibleAccountSectionProps> = ({
  title,
  isOpen,
  onOpenChange,
  data,
  onRowClick,
  t,
  loading,
}) => (
  <Collapsible open={isOpen} onOpenChange={onOpenChange}>
    <CollapsibleTrigger className="flex items-center space-x-1 font-semibold">
      {isOpen ? <IoCaretDownSharp /> : <IoCaretForwardSharp />}
      <span>{title}</span>
    </CollapsibleTrigger>
    <CollapsibleContent className="max-w-[864px] pt-2">
      <DataTable
        onRowClick={onRowClick}
        cellClassName="py-0! h-10!"
        data={data}
        columns={[
          {
            key: "code",
            header: t("details.code"),
            render: (code) => <span className="font-mono text-xs font-bold">{code}</span>,
          },
          { key: "name", header: t("details.name") },
        ]}
        loading={loading}
      />
    </CollapsibleContent>
  </Collapsible>
)
