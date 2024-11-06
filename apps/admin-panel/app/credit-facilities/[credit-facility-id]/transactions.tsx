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
  GetCreditFacilityDetailsQuery,
  CreditFacilityHistoryEntry,
  CollateralAction,
} from "@/lib/graphql/generated"
import {
  formatCollateralAction,
  formatCollateralizationState,
  formatDate,
  formatTransactionType,
} from "@/lib/utils"

const formatTransactionTypeWithoutPrefix = (type: string) => {
  const formattedType = formatTransactionType(type)
  return formattedType.replace("Credit Facility", "").trim()
}

type CreditFacilityTransactionsProps = {
  creditFacility: NonNullable<GetCreditFacilityDetailsQuery["creditFacility"]>
}

export const CreditFacilityTransactions: React.FC<CreditFacilityTransactionsProps> = ({
  creditFacility,
}) => {
  return (
    <Card className="mt-4">
      <CardHeader>
        <CardTitle>Transactions</CardTitle>
      </CardHeader>
      <CardContent>
        {creditFacility.transactions.length !== 0 ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Transaction Type</TableHead>
                <TableHead>Transaction Id</TableHead>
                <TableHead>Recorded At</TableHead>
                <TableHead className="text-right">Amount</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>{creditFacility.transactions.map(renderTransactionRow)}</TableBody>
          </Table>
        ) : (
          <p>No transactions found</p>
        )}
      </CardContent>
    </Card>
  )
}

const renderTransactionRow = (transaction: CreditFacilityHistoryEntry, index: number) => {
  const renderAmount = () => {
    switch (transaction.__typename) {
      case "CreditFacilityCollateralUpdated":
        return (
          <div
            className={`flex justify-end gap-1 ${
              transaction.action === CollateralAction.Add
                ? "text-success"
                : "text-destructive"
            }`}
          >
            <div>{transaction.action === CollateralAction.Add ? "+" : "-"}</div>
            <Balance amount={transaction.satoshis} currency="btc" align="end" />
          </div>
        )
      case "CreditFacilityCollateralizationUpdated":
        return (
          <div className="flex flex-col gap-1 justify-end">
            <Balance amount={transaction.collateral} currency="btc" align="end" />
          </div>
        )
      case "CreditFacilityOrigination":
      case "CreditFacilityIncrementalPayment":
      case "CreditFacilityDisbursalExecuted":
        return <Balance amount={transaction.cents} currency="usd" align="end" />
      default:
        return <span>-</span>
    }
  }

  const renderTransactionType = () => {
    if (!transaction.__typename) return <div>Unknown Transaction Type</div>

    switch (transaction.__typename) {
      case "CreditFacilityCollateralUpdated":
        return (
          <div className="flex flex-row gap-1">
            <div>{formatTransactionTypeWithoutPrefix(transaction.__typename)}</div>
            <div className="text-textColor-secondary text-sm">
              {formatCollateralAction(transaction.action)}
            </div>
          </div>
        )
      case "CreditFacilityCollateralizationUpdated":
        return (
          <div className="flex flex-row gap-1">
            <div>{formatTransactionTypeWithoutPrefix(transaction.__typename)}</div>
            <div className="text-textColor-secondary text-sm">
              ({formatCollateralizationState(transaction.state)})
            </div>
          </div>
        )
      default:
        return <div>{formatTransactionTypeWithoutPrefix(transaction.__typename)}</div>
    }
  }

  const getTxId = (transaction: CreditFacilityHistoryEntry): string | undefined => {
    if ("txId" in transaction) {
      return transaction.txId
    }
    return undefined
  }

  return (
    <TableRow key={index}>
      <TableCell>{renderTransactionType()}</TableCell>
      <TableCell>{getTxId(transaction) || "-"}</TableCell>
      <TableCell>
        {transaction.recordedAt ? formatDate(transaction.recordedAt) : "-"}
      </TableCell>
      <TableCell className="text-right">{renderAmount()}</TableCell>
    </TableRow>
  )
}
