"use client"

import { gql } from "@apollo/client"
import { useRouter } from "next/navigation"

import { useGetWithdrawalDetailsQuery } from "@/lib/graphql/generated"
import { DetailItem } from "@/components/details"
import { LoanBadge } from "@/components/loan/loan-badge"
import { Card, CardContent, CardHeader } from "@/components/primitive/card"
import { Separator } from "@/components/primitive/separator"
import { Button } from "@/components/primitive/button"

gql`
  query GetWithdrawalDetails($id: UUID!) {
    withdrawal(id: $id) {
      customerId
      withdrawalId
      amount
      status
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
  } = useGetWithdrawalDetailsQuery({
    variables: { id: withdrawalId },
  })

  return (
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
              <div className="grid auto-rows-min ">
                <DetailItem
                  label="Customer ID"
                  value={withdrawalDetails.withdrawal.customerId}
                />
              </div>
            </div>
            <Button
              onClick={() =>
                router.push(`/customer/${withdrawalDetails.withdrawal?.customerId}`)
              }
              className="mt-2"
            >
              Show Customer
            </Button>
          </CardContent>
        </>
      ) : (
        withdrawalId &&
        !withdrawalDetails?.withdrawal?.withdrawalId && (
          <CardContent className="pt-6">No withdrawal found with this ID</CardContent>
        )
      )}
    </Card>
  )
}

export default WithdrawalDetailsCard
