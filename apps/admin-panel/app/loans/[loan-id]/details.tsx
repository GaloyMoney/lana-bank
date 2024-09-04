"use client"

import { gql } from "@apollo/client"
import { useEffect, useState } from "react"

import Link from "next/link"

import { CollateralUpdateDialog } from "../update-collateral"
import { CollateralizationStateUpdateDialog } from "../update-collateralization-state"
import { LoanStatusBadge } from "../status-badge"
import { LoanPartialPaymentDialog } from "../partial-payment"
import { LoanApproveDialog } from "../approve"

import Balance from "@/components/balance/balance"
import { DetailItem } from "@/components/details"
import { Button } from "@/components/primitive/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/primitive/card"
import { Separator } from "@/components/primitive/separator"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"

import {
  LoanStatus,
  useGetLoanDetailsQuery,
  Loan,
  LoanHistory,
  LoanCollaterizationState,
  useGetRealtimePriceUpdatesQuery,
  CollateralAction,
} from "@/lib/graphql/generated"
import { formatInterval, formatPeriod, currencyConverter, formatDate } from "@/lib/utils"

gql`
  query GetLoanDetails($id: UUID!) {
    loan(id: $id) {
      id
      loanId
      createdAt
      approvedAt
      principal
      expiresAt
      status
      collateralizationState
      customer {
        customerId
        email
      }
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
      transactions {
        ... on IncrementalPayment {
          cents
          recordedAt
          txId
        }
        ... on InterestAccrued {
          cents
          recordedAt
          txId
        }
        ... on CollateralUpdated {
          satoshis
          recordedAt
          action
          txId
        }
        ... on LoanOrigination {
          cents
          recordedAt
          txId
        }
        ... on CollateralizationUpdated {
          state
          outstandingPrincipal
          outstandingInterest
          price
          collateral
          recordedAt
        }
      }
      loanTerms {
        annualRate
        interval
        liquidationCvl
        marginCallCvl
        initialCvl
        duration {
          period
          units
        }
      }
      currentCvl @client
      collateralToMatchInitialCvl @client
    }
  }
`

type LoanDetailsProps = { loanId: string }

const LoanDetails: React.FC<LoanDetailsProps> = ({ loanId }) => {
  const [openCollateralizationStateDialog, setOpenCollateralizationStateDialog] =
    useState<boolean>(false)
  const [openCollateralUpdateDialog, setOpenCollateralUpdateDialog] =
    useState<boolean>(false)

  const { data: priceInfo } = useGetRealtimePriceUpdatesQuery({
    fetchPolicy: "cache-only",
  })

  const {
    data: loanDetails,
    loading,
    error,
    refetch,
  } = useGetLoanDetailsQuery({ variables: { id: loanId } })

  const MarginCallPrice = calculatePrice({
    cvlPercentage: loanDetails?.loan?.loanTerms.marginCallCvl,
    principalInCents: loanDetails?.loan?.principal,
    collateralInSatoshis: loanDetails?.loan?.balance.collateral.btcBalance,
  })

  const LiquidationCallPrice = calculatePrice({
    cvlPercentage: loanDetails?.loan?.loanTerms.liquidationCvl,
    principalInCents: loanDetails?.loan?.principal,
    collateralInSatoshis: loanDetails?.loan?.balance.collateral.btcBalance,
  })

  // If price changes, refetch current CVL
  useEffect(() => {
    refetch()
  }, [priceInfo?.realtimePrice.usdCentsPerBtc, refetch])

  const formatTransactionType = (typename: string) => {
    return typename
      .replace(/([a-z])([A-Z])/g, "$1 $2")
      .replace(/^\w/, (c) => c.toUpperCase())
  }

  const renderTransactionRow = (transaction: LoanHistory) => {
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

  return (
    <>
      {loanDetails && loanDetails.loan?.loanId && (
        <>
          <CollateralUpdateDialog
            setOpenCollateralUpdateDialog={setOpenCollateralUpdateDialog}
            openCollateralUpdateDialog={openCollateralUpdateDialog}
            loanData={{
              loanId: loanDetails.loan?.loanId,
              existingCollateral: loanDetails.loan?.balance.collateral.btcBalance,
            }}
            refetch={refetch}
          />
          <CollateralizationStateUpdateDialog
            setOpenDialog={setOpenCollateralizationStateDialog}
            openDialog={openCollateralizationStateDialog}
            loanData={{
              loanId: loanDetails.loan?.loanId,
              currentState: formatCollateralizationState(
                loanDetails.loan?.collateralizationState,
              ),
            }}
            refetch={refetch}
          />
        </>
      )}
      <Card>
        {loading ? (
          <CardContent className="pt-6">Loading...</CardContent>
        ) : error ? (
          <CardContent className="pt-6 text-destructive">{error.message}</CardContent>
        ) : loanDetails?.loan ? (
          <>
            <CardHeader className="flex flex-row justify-between items-center">
              <div>
                <h2 className="font-semibold leading-none tracking-tight">Loan</h2>
                <p className="text-textColor-secondary text-sm mt-2">
                  {loanDetails.loan.loanId}
                </p>
              </div>
              <div className="flex flex-col gap-2">
                <LoanStatusBadge status={loanDetails.loan.status} className="p-1 px-4" />
              </div>
            </CardHeader>
            <Separator className="mb-6" />
            <CardContent>
              <div className="grid grid-cols-2 gap-6">
                <div className="grid auto-rows-min ">
                  <DetailItem
                    label="Customer Email"
                    value={loanDetails.loan.customer.email}
                  />
                  <DetailItem
                    label="Date created"
                    value={formatDate(loanDetails.loan.createdAt)}
                  />
                  <DetailItem
                    label="Date approved"
                    value={
                      loanDetails.loan.approvedAt
                        ? formatDate(loanDetails.loan.approvedAt)
                        : "n/a"
                    }
                  />
                  <DetailItem
                    label="Term ends"
                    value={
                      loanDetails.loan.expiresAt
                        ? formatDate(loanDetails.loan.expiresAt)
                        : "n/a"
                    }
                  />
                  <DetailItem
                    label="Principal"
                    valueComponent={
                      <Balance amount={loanDetails.loan.principal} currency="usd" />
                    }
                  />
                  <DetailItem
                    label="Duration"
                    value={`${loanDetails.loan.loanTerms.duration.units} ${formatPeriod(loanDetails.loan.loanTerms.duration.period)}`}
                  />
                  <DetailItem
                    label="Interest (APR)"
                    value={`${loanDetails.loan.loanTerms.annualRate}%`}
                  />
                  <DetailItem
                    label="Interest payment schedule"
                    value={formatInterval(loanDetails.loan.loanTerms.interval)}
                  />
                </div>
                <div className="grid auto-rows-min">
                  <DetailItem
                    label="Outstanding balance"
                    valueComponent={
                      <Balance
                        amount={loanDetails.loan.balance.outstanding.usdBalance}
                        currency="usd"
                      />
                    }
                  />
                  <DetailItem
                    label="Interest Incurred"
                    valueComponent={
                      <Balance
                        amount={loanDetails.loan.balance.interestIncurred.usdBalance}
                        currency="usd"
                      />
                    }
                  />
                  <DetailItem
                    label="Collateral balance"
                    valueComponent={
                      <Balance
                        amount={loanDetails.loan.balance.collateral.btcBalance}
                        currency="btc"
                      />
                    }
                  />

                  {loanDetails.loan.currentCvl === 0 ? (
                    <DetailItem
                      label="Expected Collateral"
                      valueComponent={
                        <Balance
                          amount={loanDetails.loan.collateralToMatchInitialCvl}
                          currency="btc"
                        />
                      }
                    />
                  ) : loanDetails.loan.currentCvl ? (
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
                      value={`${loanDetails.loan.currentCvl}%`}
                    />
                  ) : (
                    <DetailItem label="Current CVL" value="Price not available" />
                  )}
                  <DetailItem
                    label="Initial CVL"
                    value={`${loanDetails.loan.loanTerms.initialCvl}%`}
                  />
                  <DetailItem
                    labelComponent={
                      <p className="text-textColor-secondary flex items-center">
                        <div className="mr-1">Margin Call CVL</div>
                        {loanDetails.loan.balance.collateral.btcBalance > 0 ? (
                          <>
                            <div>(</div>
                            <Balance amount={MarginCallPrice} currency="usd" />
                            <div>)</div>
                          </>
                        ) : (
                          <></>
                        )}
                      </p>
                    }
                    value={`${loanDetails.loan.loanTerms.marginCallCvl}%`}
                  />

                  <DetailItem
                    labelComponent={
                      <p className="text-textColor-secondary flex items-center">
                        <div className="mr-1">Liquidation CVL</div>
                        {loanDetails.loan.balance.collateral.btcBalance > 0 ? (
                          <>
                            <div>(</div>
                            <Balance amount={LiquidationCallPrice} currency="usd" />
                            <div>)</div>
                          </>
                        ) : (
                          <></>
                        )}
                      </p>
                    }
                    value={`${loanDetails.loan.loanTerms.liquidationCvl}%`}
                  />

                  <DetailItem
                    label="Collaterization State"
                    value={formatCollateralizationState(
                      loanDetails.loan.collateralizationState,
                    )}
                  />
                </div>
              </div>
            </CardContent>
            <Separator className="mb-6" />
            <div className="flex justify-between items-center p-6 pt-0 mt-0">
              <Link
                href={`/customers/${loanDetails.loan.customer.customerId}`}
                prefetch={false}
              >
                <Button>View Customer</Button>
              </Link>
              <div className="flex flex-row gap-2">
                {loanDetails.loan.status !== LoanStatus.Closed && (
                  <Button onClick={() => setOpenCollateralUpdateDialog(true)}>
                    Update collateral
                  </Button>
                )}
                {loanDetails.loan.status === LoanStatus.Active && (
                  <LoanPartialPaymentDialog
                    refetch={refetch}
                    loanId={loanDetails.loan.loanId}
                  >
                    <Button>Make Payment</Button>
                  </LoanPartialPaymentDialog>
                )}
                {loanDetails.loan.status === LoanStatus.New && (
                  <LoanApproveDialog
                    refetch={refetch}
                    loanDetails={loanDetails.loan as Loan}
                  >
                    <Button>Approve Loan</Button>
                  </LoanApproveDialog>
                )}
                {loanDetails.loan.status === LoanStatus.Active &&
                  loanDetails.loan.collateralizationState ===
                    LoanCollaterizationState.UnderLiquidationThreshold && (
                    <Button onClick={() => setOpenCollateralizationStateDialog(true)}>
                      Update Collateralization
                    </Button>
                  )}
              </div>
            </div>
          </>
        ) : (
          loanId &&
          !loanDetails?.loan && (
            <CardContent className="pt-6">No loan found with this ID</CardContent>
          )
        )}
      </Card>
      <Card className="mt-4">
        <CardHeader>
          <CardTitle>Loan History</CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            <p>Loading...</p>
          ) : error ? (
            <p className="text-destructive">{error.message}</p>
          ) : loanDetails?.loan?.transactions.length ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Transaction Type</TableHead>
                  <TableHead>Transaction Id</TableHead>
                  <TableHead>Recorded At</TableHead>
                  <TableHead className="text-right">Amount</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {loanDetails.loan.transactions.map(renderTransactionRow)}
              </TableBody>
            </Table>
          ) : (
            <CardDescription>No transactions found</CardDescription>
          )}
        </CardContent>
      </Card>
    </>
  )
}

export default LoanDetails

export const formatCollateralizationState = (
  collateralizationState: LoanCollaterizationState,
) => {
  return collateralizationState
    .toLowerCase()
    .split("_")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ")
}

const formatCollateralAction = (collateralAction: CollateralAction) => {
  return collateralAction === CollateralAction.Add ? "(Added)" : "(Removed)"
}

const calculatePrice = ({
  cvlPercentage,
  principalInCents,
  collateralInSatoshis,
}: {
  cvlPercentage: number
  principalInCents: number
  collateralInSatoshis: number
}) => {
  return (
    (cvlPercentage * currencyConverter.centsToUsd(principalInCents)) /
    currencyConverter.satoshiToBtc(collateralInSatoshis)
  )
}
