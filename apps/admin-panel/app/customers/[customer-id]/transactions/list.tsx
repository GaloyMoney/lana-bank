"use client"

import React from "react"

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/ui/card"
import Balance from "@/components/balance/balance"
import DataTable, { Column } from "@/components/data-table"
import { GetCustomerTransactionsQuery } from "@/lib/graphql/generated"
import { formatDate } from "@/lib/utils"
import { WithdrawalStatusBadge } from "@/app/withdrawals/status-badge"
import { UsdCents } from "@/types"

type Deposit = NonNullable<
  GetCustomerTransactionsQuery["customer"]
>["depositAccount"]["deposits"][number]
type Withdrawal = NonNullable<
  GetCustomerTransactionsQuery["customer"]
>["depositAccount"]["withdrawals"][number]
type Transaction = Deposit | Withdrawal

const isWithdrawal = (transaction: Transaction): transaction is Withdrawal => {
  return "withdrawalId" in transaction
}

type CustomerTransactionsTableProps = {
  transactions: Transaction[]
}

export const CustomerTransactionsTable: React.FC<CustomerTransactionsTableProps> = ({
  transactions,
}) => {
  const columns: Column<Transaction>[] = [
    {
      key: "createdAt",
      header: "Date",
      render: (value: string) => formatDate(value, { includeTime: false }),
    },
    {
      key: "reference",
      header: "Type",
      render: (_: string, record: Transaction) =>
        isWithdrawal(record) ? "Withdrawal" : "Deposit",
    },
    {
      key: "amount",
      header: "Amount",
      render: (value: UsdCents) => <Balance amount={value} currency="usd" />,
    },
    {
      key: "reference",
      header: "Status",
      render: (_: string, record: Transaction) =>
        isWithdrawal(record) ? <WithdrawalStatusBadge status={record.status} /> : "N/A",
    },
  ] as const

  const getNavigateUrl = (record: Transaction) => {
    if (isWithdrawal(record)) {
      return `/withdrawals/${record.withdrawalId}`
    }
    return null
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Transactions</CardTitle>
        <CardDescription>Transactions for this Customer</CardDescription>
      </CardHeader>
      <CardContent>
        <DataTable
          data={transactions}
          columns={columns}
          emptyMessage="No data to display"
          navigateTo={getNavigateUrl}
          className="w-full table-fixed"
          headerClassName="bg-secondary [&_tr:hover]:!bg-secondary"
        />
      </CardContent>
    </Card>
  )
}
