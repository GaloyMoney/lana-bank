import { gql } from "@apollo/client"

import { toast } from "sonner"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/primitive/dialog"
import {
  Loan,
  useGetRealtimePriceUpdatesQuery,
  useLoanApproveMutation,
} from "@/lib/graphql/generated"
import { Button } from "@/components/primitive/button"
import { DetailItem, DetailsGroup } from "@/components/details"
import Balance from "@/components/balance/balance"
import { formatCurrency } from "@/lib/utils"

gql`
  mutation LoanApprove($input: LoanApproveInput!) {
    loanApprove(input: $input) {
      loan {
        id
        loanId
        createdAt
        balance {
          collateral {
            btcBalance
          }
          outstanding {
            usdBalance
          }
          interestIncurred {
            usdBalance
          }
        }
      }
    }
  }
`

export const LoanApproveDialog = ({
  loanDetails,
  children,
  refetch,
}: {
  loanDetails: Loan
  children: React.ReactNode
  refetch?: () => void
}) => {
  const { data: priceInfo } = useGetRealtimePriceUpdatesQuery()
  const [LoanApprove, { data, loading, error, reset }] = useLoanApproveMutation()

  const handleLoanApprove = async () => {
    try {
      await LoanApprove({
        variables: {
          input: {
            loanId: loanDetails.loanId,
          },
        },
      })
      toast.success("Loan Approved successfully")
      if (refetch) refetch()
    } catch (err) {
      console.error(err)
    }
  }

  const collateralRequired =
    Math.abs(
      (((((loanDetails.loanTerms.initialCvl - loanDetails.currentCvl) / 100) *
        loanDetails.principal) /
        100) *
        priceInfo?.realtimePrice.usdCentsPerBtc) /
        100,
    ) / 100_000_000

  return (
    <Dialog
      onOpenChange={(isOpen) => {
        if (!isOpen) {
          reset()
        }
      }}
    >
      <DialogTrigger asChild>{children}</DialogTrigger>
      {data ? (
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Loan Approved</DialogTitle>
            <DialogDescription>Loan Details.</DialogDescription>
          </DialogHeader>
          <DetailsGroup>
            <DetailItem label="Loan ID" value={data.loanApprove.loan.loanId} />
            <DetailItem label="Created At" value={data.loanApprove.loan.createdAt} />
            <DetailItem
              label="Collateral"
              valueComponent={
                <Balance
                  amount={data.loanApprove.loan.balance.collateral.btcBalance}
                  currency="btc"
                />
              }
            />
            <DetailItem
              label="Interest Incurred"
              valueComponent={
                <Balance
                  amount={data.loanApprove.loan.balance.interestIncurred.usdBalance}
                  currency="usd"
                />
              }
            />
            <DetailItem
              label="Outstanding"
              valueComponent={
                <Balance
                  amount={data.loanApprove.loan.balance.outstanding.usdBalance}
                  currency="usd"
                />
              }
            />
          </DetailsGroup>
        </DialogContent>
      ) : (
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Approve Loan</DialogTitle>
            <DialogDescription>Fill in the details to Approve loan.</DialogDescription>
          </DialogHeader>
          <DetailsGroup>
            <DetailItem
              label="Collateral Balance"
              valueComponent={
                <Balance
                  amount={loanDetails.balance.collateral.btcBalance}
                  currency="btc"
                />
              }
            />
            <DetailItem
              label={`Current CVL (BTC/USD: ${formatCurrency({
                amount: priceInfo?.realtimePrice.usdCentsPerBtc / 100,
                currency: "USD",
              })})`}
              value={`${loanDetails.currentCvl}%`}
            />
            <DetailItem
              label="Target (Initial) CVL %"
              value={`${loanDetails.loanTerms.initialCvl}%`}
            />
            <DetailItem
              label="Collateral to meet target CVL"
              valueComponent={<Balance amount={collateralRequired} currency="btc" />}
            />
          </DetailsGroup>
          {error && <span className="text-destructive">{error.message}</span>}
          <DialogFooter className="mt-4">
            <Button className="w-32" disabled={loading} onClick={handleLoanApprove}>
              Approve Loan
            </Button>
          </DialogFooter>
        </DialogContent>
      )}
    </Dialog>
  )
}
