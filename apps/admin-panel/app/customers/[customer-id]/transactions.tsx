"use client"

import Link from "next/link"
import { FaExternalLinkAlt } from "react-icons/fa"

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

import { GetCustomerQuery } from "@/lib/graphql/generated"
import { formatDate } from "@/lib/utils"

type CustomerTransactionsTableProps = {
  transactions: NonNullable<GetCustomerQuery["customer"]>["transactions"]
}

export const CustomerTransactionsTable: React.FC<CustomerTransactionsTableProps> = ({
  transactions,
}) => (
  <Card className="mt-4">
    <CardHeader>
      <CardTitle>Transactions</CardTitle>
    </CardHeader>
    {transactions.length === 0 ? (
      <CardContent>No transactions found for this customer</CardContent>
    ) : (
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Date</TableHead>
              <TableHead>Type</TableHead>
              <TableHead>ID</TableHead>
              <TableHead>Reference</TableHead>
              <TableHead>Amount</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Action</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {transactions.map((tx) => {
              const isDeposit = "depositId" in tx
              const id = isDeposit ? tx.depositId : tx.withdrawalId

              return (
                <TableRow key={id}>
                  <TableCell>{formatDate(tx.createdAt)}</TableCell>
                  <TableCell>{tx.__typename}</TableCell>
                  <TableCell>{id}</TableCell>
                  <TableCell>{tx.reference === id ? "" : tx.reference}</TableCell>
                  <TableCell className="text-right">
                    <Balance amount={tx.amount} currency={"usd"} />
                  </TableCell>
                  <TableCell>
                    {isDeposit ? "n/a" : tx.status.toLocaleLowerCase()}
                  </TableCell>
                  <TableCell>
                    <Link href={isDeposit ? `/deposits` : `/withdrawals/${id}`}>
                      <FaExternalLinkAlt className="text-primary" />
                    </Link>
                  </TableCell>
                </TableRow>
              )
            })}
          </TableBody>
        </Table>
      </CardContent>
    )}
  </Card>
)
