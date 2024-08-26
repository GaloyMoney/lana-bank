"use client"

import { useState } from "react"
import { gql } from "@apollo/client"
import { useRouter } from "next/navigation"

import { useGetWithdrawalDetailsQuery, WithdrawalStatus } from "@/lib/graphql/generated"
import { DetailItem } from "@/components/details"
import { LoanBadge } from "@/components/loan/loan-badge"
import { Card, CardContent, CardHeader } from "@/components/primitive/card"
import { Separator } from "@/components/primitive/separator"
import { Button } from "@/components/primitive/button"
import { WithdrawalConfirmDialog } from "@/components/customer/withdrawal-confirm-dialog"
import { WithdrawalCancelDialog } from "@/components/withdrawal/cancel-withdrawal-dialog"
import { currencyConverter, formatCurrency } from "@/lib/utils"

gql`
  query GetWithdrawalDetails($id: UUID!) {
    withdrawal(id: $id) {
      customerId
      withdrawalId
      amount
      status
      customer {
        email
        customerId
        applicantId
      }
    }
  }
`

type LoanDetailsProps = { withdrawalId: string }

const WithdrawalDetailsCard: React.FC<LoanDetailsProps> = ({ withdrawalId }) => {
  const router = useRouter()

  const {
    data: withdrawalDetails,
    loading,
    error,
    refetch: refetchWithdrawal,
  } = useGetWithdrawalDetailsQuery({
    variables: { id: withdrawalId },
  })

  const [openWithdrawalCancelDialog, setOpenWithdrawalCancelDialog] =
    useState<WithdrawalWithCustomer | null>(null)
  const [openWithdrawalConfirmDialog, setOpenWithdrawalConfirmDialog] =
    useState<WithdrawalWithCustomer | null>(null)

  return (
    <>
      <Card>
        {loading ? (
          <CardContent className="pt-6">Loading...</CardContent>
        ) : error ? (
          <CardContent className="pt-6 text-destructive">{error.message}</CardContent>
        ) : withdrawalDetails?.withdrawal ? (
          <>
            <CardHeader className="flex flex-row justify-between items-center">
              <div>
                <h2 className="font-semibold leading-none tracking-tight">Withdrawal</h2>
                <p className="text-textColor-secondary text-sm mt-2">
                  {withdrawalDetails.withdrawal.withdrawalId}
                </p>
              </div>
              <div className="flex flex-col gap-2">
                <LoanBadge
                  status={withdrawalDetails.withdrawal.status}
                  className="p-1 px-4"
                />
              </div>
            </CardHeader>
            <Separator className="mb-6" />
            <CardContent>
              <div className="grid grid-cols-2 gap-6">
                <div className="grid grid-rows-min">
                  <DetailItem
                    label="Customer ID"
                    value={withdrawalDetails.withdrawal.customerId}
                  />
                  <DetailItem
                    label="Withdrawal ID"
                    value={withdrawalDetails.withdrawal.withdrawalId}
                  />
                  <DetailItem
                    label="Withdrawal Amount"
                    value={formatCurrency({
                      amount: currencyConverter.centsToUsd(
                        withdrawalDetails.withdrawal.amount,
                      ),
                      currency: "usd",
                    })}
                  />
                </div>
              </div>
              <Separator className="my-6" />
              <div className="flex items-center justify-between">
                <Button
                  onClick={() =>
                    router.push(`/customer/${withdrawalDetails.withdrawal?.customerId}`)
                  }
                  className=""
                >
                  Show Customer
                </Button>
                <div>
                  {withdrawalDetails.withdrawal.status === WithdrawalStatus.Initiated && (
                    <Button
                      onClick={() =>
                        withdrawalDetails.withdrawal &&
                        setOpenWithdrawalConfirmDialog(withdrawalDetails.withdrawal)
                      }
                      className="ml-2"
                    >
                      Confirm
                    </Button>
                  )}
                  {withdrawalDetails.withdrawal.status === WithdrawalStatus.Initiated && (
                    <Button
                      onClick={() =>
                        withdrawalDetails.withdrawal &&
                        setOpenWithdrawalCancelDialog(withdrawalDetails.withdrawal)
                      }
                      className="ml-2"
                    >
                      Cancel
                    </Button>
                  )}
                </div>
              </div>
            </CardContent>
          </>
        ) : (
          withdrawalId &&
          !withdrawalDetails?.withdrawal?.withdrawalId && (
            <CardContent className="pt-6">No withdrawal found with this ID</CardContent>
          )
        )}
      </Card>
      {openWithdrawalConfirmDialog && (
        <WithdrawalConfirmDialog
          refetch={refetchWithdrawal}
          withdrawalData={openWithdrawalConfirmDialog}
          openWithdrawalConfirmDialog={Boolean(openWithdrawalConfirmDialog)}
          setOpenWithdrawalConfirmDialog={() => setOpenWithdrawalConfirmDialog(null)}
        />
      )}
      {openWithdrawalCancelDialog && (
        <WithdrawalCancelDialog
          refetch={refetchWithdrawal}
          withdrawalData={openWithdrawalCancelDialog}
          openWithdrawalCancelDialog={Boolean(openWithdrawalCancelDialog)}
          setOpenWithdrawalCancelDialog={() => setOpenWithdrawalCancelDialog(null)}
        />
      )}
    </>
  )
}

export default WithdrawalDetailsCard
