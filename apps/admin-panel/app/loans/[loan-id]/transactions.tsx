import React from "react"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import Balance from "@/components/balance/balance"

import {
  CollateralAction,
  GetLoanDetailsQuery,
  LoanHistoryEntry,
} from "@/lib/graphql/generated"
import {
  formatCollateralAction,
  formatCollateralizationState,
  formatDate,
  formatTransactionType,
} from "@/lib/utils"

type LoanTransactionHistoryProps = {
  loan: NonNullable<GetLoanDetailsQuery["loan"]>
}

export const LoanTransactionHistory: React.FC<LoanTransactionHistoryProps> = ({
  loan,
}) => {
  return (
    <Card className="mt-4">
      <CardHeader>
        <CardTitle>Transactions</CardTitle>
      </CardHeader>
      <CardContent>
        {loan.transactions.length !== 0 ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Transaction Type</TableHead>
                <TableHead>Transaction Id</TableHead>
                <TableHead>Recorded At</TableHead>
                <TableHead className="text-right">Amount</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>{loan.transactions.map(renderTransactionRow)}</TableBody>
          </Table>
        ) : (
          <p>No transactions found</p>
        )}
      </CardContent>
    </Card>
  )
}

const renderTransactionRow = (transaction: LoanHistoryEntry) => {
  const renderAmount = () => {
    switch (transaction.__typename) {
      case "CollateralUpdated":
        return (
          <div className="flex justify-end gap-1">
            <div>{transaction.action === CollateralAction.Add ? "+" : "-"}</div>
            <Balance amount={transaction.satoshis} currency="btc" align="end" />
          </div>
        )
      case "CollateralizationUpdated":
        return (
          <div className="flex flex-col gap-1 justify-end">
            <Balance amount={transaction.collateral} currency="btc" align="end" />
          </div>
        )
      case "LoanOrigination":
      case "IncrementalPayment":
      case "InterestAccrued":
        return <Balance amount={transaction.cents} currency="usd" align="end" />
      default:
        return <span>-</span>
    }
  }

  const renderTransactionType = () => {
    switch (transaction.__typename) {
      case "CollateralUpdated":
        return (
          <div className="flex flex-row gap-1">
            <div>{formatTransactionType(transaction.__typename)}</div>
            <div className="text-textColor-secondary text-sm">
              {formatCollateralAction(transaction.action)}
            </div>
          </div>
        )
      case "CollateralizationUpdated":
        return (
          <div className="flex flex-row gap-1">
            <div>{formatTransactionType(transaction.__typename)}</div>
            <div className="text-textColor-secondary text-sm">
              ({formatCollateralizationState(transaction.state)})
            </div>
          </div>
        )
      default:
        return <div>{formatTransactionType(transaction.__typename || "-")}</div>
    }
  }

  const recordedAt = "recordedAt" in transaction ? transaction.recordedAt : undefined
  const txId = "txId" in transaction ? transaction.txId : undefined

  return (
    <TableRow>
      <TableCell>{renderTransactionType()}</TableCell>
      <TableCell>{txId || "-"}</TableCell>
      <TableCell>{recordedAt ? formatDate(recordedAt) : "-"}</TableCell>
      <TableCell className="text-right">{renderAmount()}</TableCell>
    </TableRow>
  )
}
