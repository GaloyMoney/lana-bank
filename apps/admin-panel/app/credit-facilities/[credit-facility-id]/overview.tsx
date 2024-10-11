import React from "react"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import { DetailItem, DetailsGroup } from "@/components/details"
import Balance from "@/components/balance/balance"

import {
  CreditFacilityStatus,
  DisbursementStatus,
  GetCreditFacilityDetailsQuery,
  useGetRealtimePriceUpdatesQuery,
} from "@/lib/graphql/generated"
import { CENTS_PER_USD, SATS_PER_BTC, formatDate } from "@/lib/utils"

type CreditFacilityOverviewProps = {
  creditFacility: NonNullable<GetCreditFacilityDetailsQuery["creditFacility"]>
}

export const CreditFacilityOverview: React.FC<CreditFacilityOverviewProps> = ({
  creditFacility,
}) => {
  const { data: priceInfo } = useGetRealtimePriceUpdatesQuery({
    fetchPolicy: "cache-only",
  })

  const basisAmountInCents = calculateBaseAmountInCents(creditFacility)

  const MarginCallPrice = calculatePrice({
    cvlPercentage: creditFacility.creditFacilityTerms.marginCallCvl,
    basisAmountInCents: basisAmountInCents,
    collateralInSatoshis: creditFacility.collateral,
  })

  const LiquidationCallPrice = calculatePrice({
    cvlPercentage: creditFacility.creditFacilityTerms.liquidationCvl,
    basisAmountInCents: basisAmountInCents,
    collateralInSatoshis: creditFacility.collateral,
  })

  return (
    <Card className="mt-4">
      <CardHeader>
        <CardTitle>Credit Facility Overview</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-2 gap-6">
          <div className="grid auto-rows-min">
            <DetailsGroup>
              <DetailItem
                label="Collateral balance"
                valueComponent={
                  <Balance amount={creditFacility.collateral} currency="btc" />
                }
              />
              <DetailItem
                label={`Collateral to reach target (${creditFacility.creditFacilityTerms.initialCvl}%)`}
                valueComponent={
                  <Balance
                    amount={creditFacility.collateralToMatchInitialCvl}
                    currency="btc"
                  />
                }
              />
              {creditFacility.collateral > 0 ? (
                <>
                  <DetailItem
                    label={`Margin Call Price BTC/USD (${creditFacility.creditFacilityTerms.marginCallCvl}%)`}
                    valueComponent={<Balance amount={MarginCallPrice} currency="usd" />}
                  />
                  <DetailItem
                    label={`Liquidation Call Price BTC/USD (${creditFacility.creditFacilityTerms.liquidationCvl}%)`}
                    valueComponent={
                      <Balance amount={LiquidationCallPrice} currency="usd" />
                    }
                  />
                </>
              ) : (
                <>
                  <DetailItem
                    label="Margin Call CVL"
                    value={`${creditFacility.creditFacilityTerms.marginCallCvl}%`}
                  />
                  <DetailItem
                    label="Liquidation Call CVL"
                    value={`${creditFacility.creditFacilityTerms.liquidationCvl}%`}
                  />
                </>
              )}
            </DetailsGroup>
          </div>
          <div className="grid auto-rows-min">
            <DetailsGroup>
              <DetailItem
                label="Outstanding Balance"
                valueComponent={
                  <Balance
                    amount={creditFacility.balance.outstanding.usdBalance}
                    currency="usd"
                  />
                }
              />
              <DetailItem
                labelComponent={
                  <p className="text-textColor-secondary flex items-center">
                    <div className="mr-2">
                      Current CVL % <span className="text-sm">(BTC/USD:</span>
                    </div>
                    <Balance
                      className="text-sm"
                      amount={priceInfo?.realtimePrice.usdCentsPerBtc}
                      currency="usd"
                    />
                    <div className="text-sm">)</div>
                  </p>
                }
                value={`${creditFacility.currentCvl}%`}
              />
              {creditFacility.expiresAt && (
                <DetailItem
                  label="Expires at"
                  valueComponent={formatDate(creditFacility.expiresAt)}
                />
              )}
            </DetailsGroup>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

const calculatePrice = ({
  cvlPercentage,
  basisAmountInCents,
  collateralInSatoshis,
}: {
  cvlPercentage: number
  basisAmountInCents: number
  collateralInSatoshis: number
}) => {
  if (collateralInSatoshis === 0) return 0
  const cvlDecimal = cvlPercentage / 100
  const basisAmountUsd = basisAmountInCents / CENTS_PER_USD
  const collateralBtc = collateralInSatoshis / SATS_PER_BTC
  const priceUsd = (cvlDecimal * basisAmountUsd) / collateralBtc
  const priceInCents = priceUsd * CENTS_PER_USD
  return priceInCents
}

export const calculateBaseAmountInCents = ({
  status,
  facilityAmount,
  disbursements,
  balance,
}: {
  status: CreditFacilityStatus
  facilityAmount: number
  disbursements: { status: DisbursementStatus }[]
  balance: { outstanding: { usdBalance: number } }
}) => {
  if (status === CreditFacilityStatus.New) {
    return facilityAmount
  }

  if (status === CreditFacilityStatus.Active) {
    const hasApprovedDisbursements = disbursements.some(
      (d) => d.status === DisbursementStatus.Approved,
    )
    return hasApprovedDisbursements ? balance.outstanding.usdBalance : facilityAmount
  }

  return 0
}
