"use client"

import React from "react"
import { useTranslations } from "next-intl"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import Balance from "@/components/balance/balance"
import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/paginated-table"
import { GetDepositAccountDetailsQuery } from "@/lib/graphql/generated"
import { WithdrawalStatusBadge } from "@/app/withdrawals/status-badge"
import { DepositStatusBadge } from "@/app/deposits/status-badge"

type HistoryNode = NonNullable<
  NonNullable<GetDepositAccountDetailsQuery["depositAccountByPublicId"]>["history"]
>["edges"][number]["node"]

type HistoryConnection = NonNullable<
  GetDepositAccountDetailsQuery["depositAccountByPublicId"]
>["history"]

// TEMP FIX: for unknown entries
type SupportedHistoryNode = Extract<
  HistoryNode,
  | { __typename: "DepositEntry" }
  | { __typename: "WithdrawalEntry" }
  | { __typename: "CancelledWithdrawalEntry" }
  | { __typename: "DisbursalEntry" }
  | { __typename: "PaymentEntry" }
  | { __typename: "FreezeEntry" }
  | { __typename: "UnfreezeEntry" }
>

const isSupportedEntry = (node: HistoryNode): node is SupportedHistoryNode => {
  return (
    node.__typename === "DepositEntry" ||
    node.__typename === "WithdrawalEntry" ||
    node.__typename === "CancelledWithdrawalEntry" ||
    node.__typename === "DisbursalEntry" ||
    node.__typename === "PaymentEntry" ||
    node.__typename === "FreezeEntry" ||
    node.__typename === "UnfreezeEntry"
  )
}

export const DepositAccountTransactionsTable: React.FC<{
  history: HistoryConnection
  loading: boolean
  fetchMore: (variables: { variables: { after: string } }) => Promise<unknown>
}> = ({ history, loading, fetchMore }) => {
  const t = useTranslations("DepositAccounts.DepositAccountDetails.transactions")

  const filteredHistory = {
    ...history,
    edges: history.edges.filter((edge) => isSupportedEntry(edge.node)),
  }

  const columns: Column<SupportedHistoryNode>[] = [
    {
      key: "__typename",
      label: t("table.headers.date"),
      render: (_: HistoryNode["__typename"], entry: { recordedAt: string }) => {
        if (!entry.recordedAt) return "-"
        return <DateWithTooltip value={entry.recordedAt} />
      },
    },
    {
      key: "__typename",
      label: t("table.headers.type"),
      render: (type: SupportedHistoryNode["__typename"]) => {
        switch (type) {
          case "DepositEntry":
            return t("table.types.deposit")
          case "WithdrawalEntry":
          case "CancelledWithdrawalEntry":
            return t("table.types.withdrawal")
          case "DisbursalEntry":
            return t("table.types.disbursal")
          case "PaymentEntry":
            return t("table.types.payment")
          case "FreezeEntry":
            return t("table.types.freeze")
          case "UnfreezeEntry":
            return t("table.types.unfreeze")
          default:
            exhaustiveCheck(type)
        }
      },
    },
    {
      key: "__typename",
      label: t("table.headers.amount"),
      render: (_: SupportedHistoryNode["__typename"], entry: SupportedHistoryNode) => {
        switch (entry.__typename) {
          case "DepositEntry":
            return <Balance amount={entry.deposit.amount} currency="usd" />
          case "WithdrawalEntry":
          case "CancelledWithdrawalEntry":
            return <Balance amount={entry.withdrawal.amount} currency="usd" />
          case "DisbursalEntry":
            return "-"
          case "PaymentEntry":
            return "-"
          case "FreezeEntry":
          case "UnfreezeEntry":
            return <Balance amount={entry.amount} currency="usd" />
          default:
            exhaustiveCheck(entry)
        }
      },
    },
    {
      key: "__typename",
      label: t("table.headers.status"),
      render: (_: SupportedHistoryNode["__typename"], entry: SupportedHistoryNode) => {
        switch (entry.__typename) {
          case "DepositEntry":
            return <DepositStatusBadge status={entry.deposit.status} />
          case "WithdrawalEntry":
          case "CancelledWithdrawalEntry":
            return <WithdrawalStatusBadge status={entry.withdrawal.status} />
          case "DisbursalEntry":
            return "-"
          case "PaymentEntry":
          case "FreezeEntry":
          case "UnfreezeEntry":
            return "-"
          default:
            exhaustiveCheck(entry)
        }
      },
    },
  ]

  const getNavigateUrl = (entry: SupportedHistoryNode): string => {
    switch (entry.__typename) {
      case "DepositEntry":
        return `/deposits/${entry.deposit.publicId}`
      case "WithdrawalEntry":
      case "CancelledWithdrawalEntry":
        return `/withdrawals/${entry.withdrawal.publicId}`
      case "DisbursalEntry":
        return `/ledger-transactions/${entry.txId}`
      case "FreezeEntry":
        return `/ledger-transactions/${entry.txId}`
      case "UnfreezeEntry":
        return `/ledger-transactions/${entry.txId}`
      case "PaymentEntry":
        return ""
      default:
        return exhaustiveCheck(entry)
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <PaginatedTable<SupportedHistoryNode>
          columns={columns}
          data={filteredHistory as PaginatedData<SupportedHistoryNode>}
          loading={loading}
          fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
          pageSize={DEFAULT_PAGESIZE}
          navigateTo={getNavigateUrl}
        />
      </CardContent>
    </Card>
  )
}

const exhaustiveCheck = (value: never): never => {
  throw new Error(`Unhandled case: ${value}`)
}
