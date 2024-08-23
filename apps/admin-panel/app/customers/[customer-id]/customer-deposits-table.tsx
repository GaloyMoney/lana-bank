"use client"
import { gql } from "@apollo/client"

import { useState } from "react"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/primitive/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import { useGetDepositsForCustomerQuery } from "@/lib/graphql/generated"
import { Button } from "@/components/primitive/button"
import RecordDepositDialog from "@/components/customer/record-deposit-dialog"
import Balance from "@/components/balance/balance"

gql`
  query GetDepositsForCustomer($id: UUID!) {
    customer(id: $id) {
      customerId
      deposits {
        customerId
        depositId
        amount
      }
    }
  }
`

export const CustomerDepositsTable = ({ customerId }: { customerId: string }) => {
  const [openRecordDepositDialog, setOpenRecordDepositDialog] = useState(false)
  const { loading, error, data, refetch } = useGetDepositsForCustomerQuery({
    variables: {
      id: customerId,
    },
  })

  return (
    <>
      <RecordDepositDialog
        customerId={customerId}
        refetch={refetch}
        openRecordDepositDialog={openRecordDepositDialog}
        setOpenRecordDepositDialog={setOpenRecordDepositDialog}
      />
      <Card className="mt-4">
        {loading ? (
          <CardContent className="p-6">Loading...</CardContent>
        ) : error ? (
          <CardContent className="p-6 text-destructive">{error.message}</CardContent>
        ) : (
          <>
            <CardHeader className="flex flex-row justify-between items-center pb-0">
              <div className="flex flex-col space-y-1.5">
                <CardTitle>Deposits</CardTitle>
                <CardDescription>Deposit Details for Customer</CardDescription>
              </div>
              <Button onClick={() => setOpenRecordDepositDialog(true)}>
                New Deposit
              </Button>
            </CardHeader>
            {!data || !data.customer?.deposits || data.customer.deposits.length === 0 ? (
              <CardContent className="p-6">
                No deposits found for this customer
              </CardContent>
            ) : (
              <CardContent className="mt-6">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Deposit ID</TableHead>
                      <TableHead>Customer ID</TableHead>
                      <TableHead>Amount</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {data.customer.deposits.map((deposit) => (
                      <TableRow key={deposit.depositId}>
                        <TableCell>{deposit.depositId}</TableCell>
                        <TableCell>{deposit.customerId}</TableCell>
                        <TableCell>
                          <Balance amount={deposit.amount} currency="usd" />
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </CardContent>
            )}
          </>
        )}
      </Card>
    </>
  )
}
