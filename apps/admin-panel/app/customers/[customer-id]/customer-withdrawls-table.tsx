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
import { useGetWithdrawalsForCustomerQuery } from "@/lib/graphql/generated"
import { Button } from "@/components/primitive/button"
import WithdrawalInitiateDialog from "@/components/customer/withdrawal-initiate-dialog"
import { WithdrawalConfirmDialog } from "@/components/customer/withdrawal-confirm-dialog"

import Balance from "@/components/balance/balance"
import WithdrawalDropdown from "@/app/withdrawals/drop-down"
import { WithdrawalStatusBadge } from "@/components/withdrawal/withdrawal-status-badge"
import { WithdrawalCancelDialog } from "@/components/withdrawal/cancel-withdrawal-dialog"

gql`
  query GetWithdrawalsForCustomer($id: UUID!) {
    customer(id: $id) {
      customerId
      withdrawals {
        status
        customerId
        withdrawalId
        amount
        customer {
          customerId
          email
        }
      }
    }
  }
`

export const CustomerWithdrawalsTable = ({ customerId }: { customerId: string }) => {
  const [openWithdrawalInitiateDialog, setOpenWithdrawalInitiateDialog] = useState(false)
  const [openWithdrawalConfirmDialog, setOpenWithdrawalConfirmDialog] =
    useState<WithdrawalWithCustomer | null>(null)

  const { loading, error, data, refetch } = useGetWithdrawalsForCustomerQuery({
    variables: {
      id: customerId,
    },
  })

  const [openWithdrawalCancelDialog, setOpenWithdrawalCancelDialog] =
    useState<WithdrawalWithCustomer | null>(null)
  return (
    <>
      <WithdrawalInitiateDialog
        refetch={refetch}
        customerId={customerId}
        openWithdrawalInitiateDialog={openWithdrawalInitiateDialog}
        setOpenWithdrawalInitiateDialog={setOpenWithdrawalInitiateDialog}
      />
      {openWithdrawalConfirmDialog && (
        <WithdrawalConfirmDialog
          withdrawalData={openWithdrawalConfirmDialog}
          openWithdrawalConfirmDialog={Boolean(openWithdrawalConfirmDialog)}
          setOpenWithdrawalConfirmDialog={() => setOpenWithdrawalConfirmDialog(null)}
          refetch={refetch}
        />
      )}
      {openWithdrawalCancelDialog && (
        <WithdrawalCancelDialog
          refetch={refetch}
          withdrawalData={openWithdrawalCancelDialog}
          openWithdrawalCancelDialog={Boolean(openWithdrawalCancelDialog)}
          setOpenWithdrawalCancelDialog={() => setOpenWithdrawalCancelDialog(null)}
        />
      )}
      <Card className="mt-4">
        {loading ? (
          <CardContent className="p-6">Loading...</CardContent>
        ) : error ? (
          <CardContent className="p-6 text-destructive">{error.message}</CardContent>
        ) : (
          <>
            <CardHeader className="flex flex-row justify-between align-middle items-center pb-0">
              <div className="flex flex-col space-y-1.5">
                <CardTitle>Withdrawals</CardTitle>
                <CardDescription>Withdrawal Details for Customer</CardDescription>
              </div>
              <Button onClick={() => setOpenWithdrawalInitiateDialog(true)}>
                New Withdrawal
              </Button>
            </CardHeader>
            {!data ||
            !data.customer?.withdrawals ||
            data.customer.withdrawals.length === 0 ? (
              <CardContent className="p-6">
                No withdrawals found for this customer
              </CardContent>
            ) : (
              <CardContent className="mt-6">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Withdrawal ID</TableHead>
                      <TableHead>Amount</TableHead>
                      <TableHead>Status</TableHead>
                      <TableHead></TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {data.customer.withdrawals.map((withdrawal) => (
                      <>
                        <TableRow key={withdrawal.withdrawalId}>
                          <TableCell>{withdrawal.withdrawalId}</TableCell>
                          <TableCell>
                            <Balance amount={withdrawal.amount} currency="usd" />
                          </TableCell>
                          <TableCell>
                            <WithdrawalStatusBadge status={withdrawal.status} />
                          </TableCell>
                          <TableCell>
                            <WithdrawalDropdown
                              withdrawal={withdrawal}
                              onConfirm={() => setOpenWithdrawalConfirmDialog(withdrawal)}
                              onCancel={() => setOpenWithdrawalCancelDialog(withdrawal)}
                            />
                          </TableCell>
                        </TableRow>
                      </>
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
