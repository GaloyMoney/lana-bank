"use client"
import { useState } from "react"
import { useRouter } from "next/navigation"

import { LoanStatusBadge } from "../status-badge"
import { LoanPartialPaymentDialog } from "../partial-payment"
import { LoanApproveDialog } from "../approve"
import { CollateralUpdateDialog } from "../update-collateral"
import { CollateralizationStateUpdateDialog } from "../update-collateralization-state"

import { DetailItem, DetailsGroup } from "@/components/details"
import { Button } from "@/components/primitive/button"
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/primitive/card"
import Balance from "@/components/balance/balance"

import {
  GetLoanDetailsQuery,
  Loan,
  LoanCollaterizationState,
  LoanStatus,
} from "@/lib/graphql/generated"
import { formatCollateralizationState } from "@/lib/utils"

type LoanDetailsCardProps = {
  loan: NonNullable<GetLoanDetailsQuery["loan"]>
  refetch: () => void
}

export const LoanDetailsCard: React.FC<LoanDetailsCardProps> = ({ loan, refetch }) => {
  const [openCollateralizationStateDialog, setOpenCollateralizationStateDialog] =
    useState(false)
  const [openCollateralUpdateDialog, setOpenCollateralUpdateDialog] = useState(false)

  const router = useRouter()

  return (
    <>
      <Card>
        <CardHeader className="pb-4">
          <div className="flex justify-between items-center">
            <CardTitle>Loan Overview</CardTitle>
            <LoanStatusBadge status={loan.status} />
          </div>
        </CardHeader>
        <div className="flex w-full items-center justify-between">
          <CardContent className="flex-1">
            <DetailsGroup>
              <DetailItem label="Loan ID" value={loan.loanId} />
              <DetailItem
                label="Customer"
                value={loan.customer.email}
                onClick={() => {
                  router.push(`/customers/${loan.customer.customerId}`)
                }}
              />
              <DetailItem
                label="Principal"
                valueComponent={<Balance amount={loan.facility} currency="usd" />}
              />
              <DetailItem
                label="Collaterization State"
                value={formatCollateralizationState(loan.collateralizationState)}
              />
            </DetailsGroup>
          </CardContent>
          <CardFooter className="flex space-x-4 justify-end">
            {loan.status !== LoanStatus.Closed && (
              <Button onClick={() => setOpenCollateralUpdateDialog(true)}>
                Update collateral
              </Button>
            )}
            {loan.status === LoanStatus.Active && (
              <LoanPartialPaymentDialog refetch={refetch} loanId={loan.loanId}>
                <Button>Make Payment</Button>
              </LoanPartialPaymentDialog>
            )}
            {loan.status === LoanStatus.New && (
              <LoanApproveDialog refetch={refetch} loanDetails={loan as Loan}>
                <Button>Approve Loan</Button>
              </LoanApproveDialog>
            )}
            {loan.status === LoanStatus.Active &&
              loan.collateralizationState ===
                LoanCollaterizationState.UnderLiquidationThreshold && (
                <Button onClick={() => setOpenCollateralizationStateDialog(true)}>
                  Update Collateralization
                </Button>
              )}
          </CardFooter>
        </div>
      </Card>
      <>
        <CollateralUpdateDialog
          setOpenCollateralUpdateDialog={setOpenCollateralUpdateDialog}
          openCollateralUpdateDialog={openCollateralUpdateDialog}
          loanData={{
            loanId: loan.loanId,
            existingCollateral: loan.balance.collateral.btcBalance,
          }}
          refetch={refetch}
        />
        <CollateralizationStateUpdateDialog
          setOpenDialog={setOpenCollateralizationStateDialog}
          openDialog={openCollateralizationStateDialog}
          loanData={{
            loanId: loan.loanId,
            currentState: formatCollateralizationState(loan?.collateralizationState),
          }}
          refetch={refetch}
        />
      </>
    </>
  )
}
