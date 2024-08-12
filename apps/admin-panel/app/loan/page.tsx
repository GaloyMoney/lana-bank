"use client"
import React, { useEffect, useState } from "react"
import { useRouter, useSearchParams } from "next/navigation"

import Loans from "./loans"

import { Button } from "@/components/primitive/button"
import { Input } from "@/components/primitive/input"
import { Card, CardContent, CardHeader } from "@/components/primitive/card"
import { Separator } from "@/components/primitive/separator"
import { PageHeading } from "@/components/page-heading"
import { DetailItem } from "@/components/details"
import { currencyConverter, formatCurrency, formatDate } from "@/lib/utils"
import { LoanStatus, useGetLoanDetailsQuery } from "@/lib/graphql/generated"
import { LoanPartialPaymentDialog } from "@/components/loan/loan-partial-payment"
import { LoanApproveDialog } from "@/components/loan/approve-loan"
import { LoanBadge } from "@/components/loan/loan-badge"
import { formatInterval, formatPeriod } from "@/lib/terms/utils"

function LoanPage() {
  const searchParams = useSearchParams()
  const loanIdParam = searchParams.get("loanId")
  const router = useRouter()

  const [loanId, setLoanId] = useState(loanIdParam)
  const [inputLoanId, setInputLoanId] = useState("")

  useEffect(() => {
    setLoanId(loanIdParam)
  }, [loanIdParam])

  const {
    loading,
    error,
    data: loanDetails,
    refetch,
  } = useGetLoanDetailsQuery({
    skip: !loanId,
    variables: {
      id: loanId || "",
    },
  })

  useEffect(() => {
    if (loanId) {
      const interval = setInterval(() => {
        refetch()
      }, 10000)
      return () => clearInterval(interval)
    }
  }, [loanId, refetch])

  const handleSearch = () => {
    router.push(`?loanId=${inputLoanId}`)
  }

  return (
    <main>
      <div className="flex justify-between items-center mb-8">
        <PageHeading className="mb-0">Loan</PageHeading>
        <div className="flex gap-2">
          <Input
            onChange={(e) => setInputLoanId(e.target.value)}
            placeholder="Find a loan by loan ID"
            id="loanId"
            name="loanId"
            value={inputLoanId}
            className="w-80"
          />
          <Button onClick={handleSearch} variant="primary">
            Search
          </Button>
        </div>
      </div>

      <Card>
        {loading ? (
          <CardContent className="pt-6">Loading...</CardContent>
        ) : error ? (
          <CardContent className="pt-6 text-destructive">{error.message}</CardContent>
        ) : loanDetails?.loan ? (
          <>
            <CardHeader className="flex flex-row justify-between items-center">
              <div>
                <h2 className="font-semibold leading-none tracking-tight">
                  Loan Details
                </h2>
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
                    label="customer ID"
                    value={loanDetails.loan.customer.customerId}
                  />
                  <DetailItem
                    label="Start Date"
                    value={formatDate(loanDetails.loan.startDate)}
                  />
                  <DetailItem
                    label="Collateral balance (BTC)"
                    value={`${loanDetails.loan.balance.collateral.btcBalance} sats`}
                  />
                  <DetailItem
                    label="Outstanding balance (USD)"
                    value={formatCurrency({
                      amount: currencyConverter.centsToUsd(
                        loanDetails.loan.balance.outstanding.usdBalance,
                      ),
                      currency: "USD",
                    })}
                  />
                  <DetailItem
                    label="Interest Incurred (USD)"
                    value={formatCurrency({
                      amount: currencyConverter.centsToUsd(
                        loanDetails.loan.balance.interestIncurred.usdBalance,
                      ),
                      currency: "USD",
                    })}
                  />
                  <DetailItem
                    label="Initial CVL"
                    value={`${loanDetails.loan.loanTerms.initialCvl}%`}
                  />
                </div>
                <div className="grid auto-rows-min">
                  <DetailItem
                    label="Duration"
                    value={`${loanDetails.loan.loanTerms.duration.units} ${formatPeriod(loanDetails.loan.loanTerms.duration.period)}`}
                  />
                  <DetailItem
                    label="Interval"
                    value={formatInterval(loanDetails.loan.loanTerms.interval)}
                  />
                  <DetailItem
                    label="Annual Rate"
                    value={`${loanDetails.loan.loanTerms.annualRate}%`}
                  />
                  <DetailItem
                    label="Liquidation CVL"
                    value={`${loanDetails.loan.loanTerms.liquidationCvl}%`}
                  />
                  <DetailItem
                    label="Margin Call CVL"
                    value={`${loanDetails.loan.loanTerms.marginCallCvl}%`}
                  />
                </div>
              </div>
            </CardContent>
            {loanDetails.loan.status !== LoanStatus.Closed && (
              <Separator className="mb-6" />
            )}
            <div className="flex flex-row gap-2 p-6 pt-0 mt-0">
              {loanDetails.loan.status === LoanStatus.Active && (
                <LoanPartialPaymentDialog
                  refetch={refetch}
                  loanId={loanDetails.loan.loanId}
                >
                  <Button>Make Payment</Button>
                </LoanPartialPaymentDialog>
              )}
              {loanDetails.loan.status === LoanStatus.New && (
                <LoanApproveDialog refetch={refetch} loanId={loanDetails.loan.loanId}>
                  <Button>Approve Loan</Button>
                </LoanApproveDialog>
              )}
            </div>
          </>
        ) : (
          loanId &&
          !loanDetails?.loan && (
            <CardContent className="pt-6">No loan found with this ID</CardContent>
          )
        )}
      </Card>

      <Loans />
    </main>
  )
}

export default LoanPage
