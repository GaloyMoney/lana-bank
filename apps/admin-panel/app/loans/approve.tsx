import React from "react"
import { gql } from "@apollo/client"
import { FaExclamationCircle } from "react-icons/fa"
import { useSession } from "next-auth/react"

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
  LoanCollaterizationState,
  useGetRealtimePriceUpdatesQuery,
  useLoanApproveMutation,
} from "@/lib/graphql/generated"
import { Button } from "@/components/primitive/button"
import { DetailItem, DetailsGroup } from "@/components/details"
import Balance from "@/components/balance/balance"

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

type LoanApproveDialogProps = {
  loanDetails: Loan
  refetch?: () => void
}

export const LoanApproveDialog: React.FC<
  React.PropsWithChildren<LoanApproveDialogProps>
> = ({ loanDetails, children, refetch }) => {
  const { data: session } = useSession()

  const { data: priceInfo } = useGetRealtimePriceUpdatesQuery({
    fetchPolicy: "cache-only",
  })
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

  const hasApprovals = loanDetails.approvals.length > 0
  const noCollateral = loanDetails.balance.collateral.btcBalance === 0
  const approvedBy = loanDetails.approvals.map((a) => a.user.email).join(", ")
  const userHasAlreadyApproved = loanDetails.approvals
    .map((a) => a.user.email)
    .includes(session?.user?.email || "")
  const canApproveLoan =
    loanDetails.collateralizationState === LoanCollaterizationState.FullyCollateralized &&
    !userHasAlreadyApproved

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
              label="Expected Collateral to meet target CVL"
              valueComponent={
                <Balance
                  amount={loanDetails.collateralToMatchInitialCvl}
                  currency="btc"
                />
              }
            />
            <DetailItem
              labelComponent={
                <p className="text-textColor-secondary flex items-center">
                  <div className="mr-2">Current CVL (BTC/USD:</div>
                  <Balance
                    amount={priceInfo?.realtimePrice.usdCentsPerBtc}
                    currency="usd"
                  />
                  <div>)</div>
                </p>
              }
              value={`${loanDetails.currentCvl}%`}
            />
            <DetailItem
              label="Target (Initial) CVL %"
              value={`${loanDetails.loanTerms.initialCvl}%`}
            />
            <DetailItem
              label="Margin Call CVL %"
              value={`${loanDetails.loanTerms.marginCallCvl}%`}
            />
          </DetailsGroup>
          {error && <span className="text-destructive">{error.message}</span>}
          {loanDetails.currentCvl ? (
            loanDetails.currentCvl < loanDetails.loanTerms.marginCallCvl && (
              <span className="text-destructive flex items-center space-x-2">
                <FaExclamationCircle />
                <span>Current CVL is less than Margin Call CVL</span>
              </span>
            )
          ) : (
            <></>
          )}
          {loanDetails.currentCvl ? (
            loanDetails.currentCvl > loanDetails.loanTerms.marginCallCvl &&
            loanDetails.currentCvl < loanDetails.loanTerms.initialCvl && (
              <span className="text-warning flex items-center space-x-2">
                <FaExclamationCircle />
                <span>Current CVL is less than Target CVL</span>
              </span>
            )
          ) : (
            <></>
          )}
          {noCollateral && (
            <span className="text-destructive">Loan approval requires collateral</span>
          )}
          {userHasAlreadyApproved && (
            <span className="text-primary">You have already approved this loan</span>
          )}
          {canApproveLoan && (
            <DialogFooter
              className={`mt-4 flex items-center ${hasApprovals && "sm:justify-between"}`}
            >
              {hasApprovals && (
                <div className="text-primary">
                  Approved by <span>{approvedBy}</span>
                </div>
              )}
              <Button disabled={loading} className="w-32" onClick={handleLoanApprove}>
                Approve Loan
              </Button>
            </DialogFooter>
          )}
        </DialogContent>
      )}
    </Dialog>
  )
}
