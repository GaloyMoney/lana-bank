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
import PaginatedTable, { Column, PaginatedData } from "@/components/paginated-table"
import { GetDepositAccountDetailsQuery } from "@/lib/graphql/generated"
import { WithdrawalStatusBadge } from "@/app/withdrawals/status-badge"
import { DepositStatusBadge } from "@/app/deposits/status-badge"
import { DisbursalStatusBadge } from "@/app/disbursals/status-badge"

type HistoryNode = NonNullable<
  NonNullable<GetDepositAccountDetailsQuery["depositAccountByPublicId"]>["history"]
>["edges"][number]["node"]

type HistoryConnection = NonNullable<
  GetDepositAccountDetailsQuery["depositAccountByPublicId"]
>["history"]

type DepositAccountTransactionsTableProps = {
  history: HistoryConnection
  loading: boolean
  fetchMore: (variables: { variables: { after: string } }) => Promise<unknown>
}

export const DepositAccountTransactionsTable: React.FC<
  DepositAccountTransactionsTableProps
> = ({ history, loading, fetchMore }) => {
  const t = useTranslations("DepositAccounts.DepositAccountDetails.transactions")

  //  TEMP FIX: for unknown entries
  const filteredHistory = {
    ...history,
    edges: history.edges.filter(
      (edge) =>
        edge.node.__typename &&
        [
          "DepositEntry",
          "WithdrawalEntry",
          "CancelledWithdrawalEntry",
          "DisbursalEntry",
          "PaymentEntry",
        ].includes(edge.node.__typename),
    ),
  }

  const columns: Column<HistoryNode>[] = [
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
      render: (type: HistoryNode["__typename"]) => {
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
          default:
            return "-"
        }
      },
    },
    {
      key: "__typename",
      label: t("table.headers.amount"),
      render: (_: HistoryNode["__typename"], entry: HistoryNode) => {
        switch (entry.__typename) {
          case "DepositEntry":
            return <Balance amount={entry.deposit.amount} currency="usd" />
          case "WithdrawalEntry":
          case "CancelledWithdrawalEntry":
            return <Balance amount={entry.withdrawal.amount} currency="usd" />
          case "DisbursalEntry":
            return <Balance amount={entry.disbursal.amount} currency="usd" />
          case "PaymentEntry":
            return <Balance amount={entry.payment.amount} currency="usd" />
          default:
            return "-"
        }
      },
    },
    {
      key: "__typename",
      label: t("table.headers.status"),
      render: (_: HistoryNode["__typename"], entry: HistoryNode) => {
        switch (entry.__typename) {
          case "DepositEntry":
            return <DepositStatusBadge status={entry.deposit.status} />
          case "WithdrawalEntry":
          case "CancelledWithdrawalEntry":
            return <WithdrawalStatusBadge status={entry.withdrawal.status} />
          case "DisbursalEntry":
            return <DisbursalStatusBadge status={entry.disbursal.status} />
          default:
            return "-"
        }
      },
    },
  ]

  const getNavigateUrl = (entry: HistoryNode): string => {
    if (entry.__typename === "DepositEntry") {
      return `/deposits/${entry.deposit.publicId}`
    }
    if (
      entry.__typename === "WithdrawalEntry" ||
      entry.__typename === "CancelledWithdrawalEntry"
    ) {
      return `/withdrawals/${entry.withdrawal.publicId}`
    }
    if (entry.__typename === "DisbursalEntry") {
      return `/disbursals/${entry.disbursal.publicId}`
    }
    return ""
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <PaginatedTable<HistoryNode>
          columns={columns}
          data={filteredHistory as PaginatedData<HistoryNode>}
          loading={loading}
          fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
          pageSize={100}
          navigateTo={getNavigateUrl}
        />
      </CardContent>
    </Card>
  )
}
