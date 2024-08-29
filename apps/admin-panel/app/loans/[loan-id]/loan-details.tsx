"use client"

import { gql } from "@apollo/client"
import { useEffect, useState } from "react"

import Link from "next/link"

import Balance from "@/components/balance/balance"
import { DetailItem } from "@/components/details"
import { LoanApproveDialog } from "@/components/loan/approve-loan"
import { LoanBadge } from "@/components/loan/loan-badge"
import { LoanPartialPaymentDialog } from "@/components/loan/loan-partial-payment"
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
import { formatInterval, formatPeriod } from "@/lib/term/utils"
import { formatCurrency, formatDate } from "@/lib/utils"
import { CollateralUpdateDialog } from "@/components/loan/collateral-update-dialog"

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
    }
  }
`

type LoanDetailsProps = { loanId: string }

const LoanDetails: React.FC<LoanDetailsProps> = ({ loanId }) => {
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
              <div className="flex gap-0.5 justify-end items-center">
                <Balance amount={transaction.collateral} currency="btc" align="end" />
              </div>
              <div className="flex gap-1 justify-end text-textColor-secondary text-xs">
                <span>Interest</span>({" "}
                <Balance
                  amount={transaction.outstandingPrincipal}
                  currency="usd"
                  align="end"
                />
                )
              </div>
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
        <TableCell className="py-5">{renderTransactionType()}</TableCell>
        <TableCell>{txId || "-"}</TableCell>
        <TableCell>{recordedAt ? formatDate(recordedAt) : "-"}</TableCell>
        <TableCell className="text-right">{renderAmount()}</TableCell>
      </TableRow>
    )
  }

  return (
    <>
      {loanDetails && loanDetails.loan?.loanId && (
        <CollateralUpdateDialog
          setOpenCollateralUpdateDialog={setOpenCollateralUpdateDialog}
          openCollateralUpdateDialog={openCollateralUpdateDialog}
          loanData={{
            loanId: loanDetails.loan?.loanId,
            existingCollateral: loanDetails.loan?.balance.collateral.btcBalance,
          }}
          refetch={refetch}
        />
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
                <LoanBadge status={loanDetails.loan.status} className="p-1 px-4" />
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

                  <DetailItem
                    label={`Current CVL (BTC/USD: ${formatCurrency({
                      amount: priceInfo?.realtimePrice.usdCentsPerBtc / 100,
                      currency: "USD",
                    })})`}
                    value={`${loanDetails.loan.currentCvl}%`}
                  />
                  <DetailItem
                    label="Initial CVL"
                    value={`${loanDetails.loan.loanTerms.initialCvl}%`}
                  />
                  <DetailItem
                    label="Margin Call CVL"
                    value={`${loanDetails.loan.loanTerms.marginCallCvl}%`}
                  />
                  <DetailItem
                    label="Liquidation CVL"
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

const formatCollateralizationState = (
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
