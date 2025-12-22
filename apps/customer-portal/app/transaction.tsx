"use client"

import React from "react"

import DataTable, { Column } from "@lana/web/components/data-table"

import { Badge, BadgeProps } from "@lana/web/ui/badge"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import { DisbursalStatusBadge } from "./disbursal-badge"

import { GetTransactionHistoryQuery, WithdrawalStatus } from "@/lib/graphql/generated"

import Balance from "@/components/balance"

type HistoryNode = NonNullable<
  GetTransactionHistoryQuery["me"]["customer"]
>["depositAccount"]["history"]["edges"][number]["node"]

type CustomerTransactionsTableProps = {
  historyEntries: HistoryNode[]
}

export const CustomerTransactionsTable: React.FC<CustomerTransactionsTableProps> = ({
  historyEntries,
}) => {
  // TEMP FIX: for unknown entries
  const validEntries = historyEntries.filter(
    (entry) =>
      entry.__typename &&
      [
        "DepositEntry",
        "WithdrawalEntry",
        "CancelledWithdrawalEntry",
        "DisbursalEntry",
        "PaymentEntry",
        "FreezeEntry",
        "UnfreezeEntry",
      ].includes(entry.__typename),
  )

  const columns: Column<HistoryNode>[] = [
    {
      key: "__typename",
      header: "Date",
      render: (_: HistoryNode["__typename"], entry: { recordedAt: string }) => {
        if (!entry.recordedAt) return "-"
        return <DateWithTooltip value={entry.recordedAt} />
      },
    },
    {
      key: "__typename",
      header: "Type",
      render: (type: HistoryNode["__typename"]) => {
        switch (type) {
          case "DepositEntry":
            return "Deposit"
          case "WithdrawalEntry":
          case "CancelledWithdrawalEntry":
            return "Withdrawal"
          case "DisbursalEntry":
            return "Disbursal"
          case "PaymentEntry":
            return "Payment"
          case "FreezeEntry":
            return "Freeze"
          case "UnfreezeEntry":
            return "Unfreeze"
          default:
            return "-"
        }
      },
    },
    {
      key: "__typename",
      header: "Amount",
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
          case "FreezeEntry":
          case "UnfreezeEntry":
            return <Balance amount={entry.amount} currency="usd" />
          default:
            return "-"
        }
      },
    },
    {
      key: "__typename",
      header: "Status",
      render: (_: HistoryNode["__typename"], entry: HistoryNode) => {
        switch (entry.__typename) {
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

  const getNavigateUrl = (entry: HistoryNode): string | null => {
    switch (entry.__typename) {
      default:
        return null
    }
  }

  return (
    <DataTable
      data={validEntries}
      columns={columns}
      emptyMessage={
        <div className="min-h-40 w-full border rounded-md flex items-center justify-center">
          No Transactions Found
        </div>
      }
      navigateTo={getNavigateUrl}
      className="w-full table-fixed"
      headerClassName="bg-secondary [&_tr:hover]:bg-secondary!"
    />
  )
}

interface StatusBadgeProps extends BadgeProps {
  status: WithdrawalStatus
}

const getVariant = (status: WithdrawalStatus) => {
  switch (status) {
    case WithdrawalStatus.PendingApproval:
      return "default"
    case WithdrawalStatus.Confirmed:
      return "success"
    case WithdrawalStatus.Cancelled:
      return "destructive"
    case WithdrawalStatus.Denied:
      return "destructive"
    default:
      return "default"
  }
}

export const WithdrawalStatusBadge: React.FC<StatusBadgeProps> = ({ status }) => {
  const variant = getVariant(status)
  return <Badge variant={variant}>{status.split("_").join(" ")}</Badge>
}
