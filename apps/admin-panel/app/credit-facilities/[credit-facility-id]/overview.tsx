import React from "react"
import { FaBan, FaCheckCircle, FaQuestion } from "react-icons/fa"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import { DetailItem, DetailsGroup } from "@/components/details"
import Balance from "@/components/balance/balance"
import {
  ApprovalProcessStatus,
  CreditFacilityStatus,
  DisbursalStatus,
  GetCreditFacilityDetailsQuery,
  useGetRealtimePriceUpdatesQuery,
} from "@/lib/graphql/generated"
import { CENTS_PER_USD, SATS_PER_BTC, formatDate, formatRole } from "@/lib/utils"

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
    <>
      <Card className="mt-4">
        <CardHeader>
          <CardTitle>Credit Facility Overview</CardTitle>
        </CardHeader>
        <CardContent>
          <DetailsGroup className="grid grid-cols-2 gap-x-6">
            <DetailItem
              label="Collateral balance"
              value={
                <Balance
                  amount={creditFacility.balance.collateral.btcBalance}
                  currency="btc"
                />
              }
            />
            <DetailItem
              label={`Collateral to reach target (${creditFacility.creditFacilityTerms.initialCvl}%)`}
              value={
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
                  value={<Balance amount={MarginCallPrice} currency="usd" />}
                />
                <DetailItem
                  label={`Liquidation Call Price BTC/USD (${creditFacility.creditFacilityTerms.liquidationCvl}%)`}
                  value={<Balance amount={LiquidationCallPrice} currency="usd" />}
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
            <DetailItem
              label={
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
              value={`${creditFacility.currentCvl.total}%`}
            />

            {creditFacility.expiresAt && (
              <DetailItem
                label="Expires at"
                value={formatDate(creditFacility.expiresAt)}
              />
            )}
            <DetailItem
              label="Facility Remaining"
              value={
                <Balance
                  amount={creditFacility.balance.facilityRemaining.usdBalance}
                  currency="usd"
                />
              }
            />
            <DetailItem
              label="Total Disbursed"
              value={
                <Balance
                  amount={creditFacility.balance.disbursed.total.usdBalance}
                  currency="usd"
                />
              }
            />
            <DetailItem
              label="Outstanding Disbursed"
              value={
                <Balance
                  amount={creditFacility.balance.disbursed.outstanding.usdBalance}
                  currency="usd"
                />
              }
            />
            <DetailItem
              label="Total Interest"
              value={
                <Balance
                  amount={creditFacility.balance.interest.total.usdBalance}
                  currency="usd"
                />
              }
            />
            <DetailItem
              label="Outstanding Interest"
              value={
                <Balance
                  amount={creditFacility.balance.interest.outstanding.usdBalance}
                  currency="usd"
                />
              }
            />
            <DetailItem
              label="Total Outstanding Balance"
              value={
                <Balance
                  amount={creditFacility.balance.outstanding.usdBalance}
                  currency="usd"
                />
              }
            />
          </DetailsGroup>
        </CardContent>
      </Card>
      {creditFacility.approvalProcess.rules.__typename === "CommitteeThreshold" && (
        <Card className="mt-4">
          <CardHeader>
            <CardTitle className="text-primary font-normal">
              Approval process decision from the{" "}
              {creditFacility.approvalProcess.rules.committee.name} Committee
            </CardTitle>
          </CardHeader>
          <CardContent>
            {creditFacility.approvalProcess.voters
              .filter((voter) => {
                if (
                  creditFacility?.approvalProcess.status ===
                    ApprovalProcessStatus.InProgress ||
                  ([
                    ApprovalProcessStatus.Approved,
                    ApprovalProcessStatus.Denied,
                  ].includes(
                    creditFacility?.approvalProcess.status as ApprovalProcessStatus,
                  ) &&
                    voter.didVote)
                ) {
                  return true
                }
                return false
              })
              .map((voter) => (
                <div key={voter.user.userId} className="flex items-center space-x-3 p-2">
                  {voter.didApprove ? (
                    <FaCheckCircle className="h-6 w-6 text-green-500" />
                  ) : voter.didDeny ? (
                    <FaBan className="h-6 w-6 text-red-500" />
                  ) : !voter.didVote ? (
                    <FaQuestion className="h-6 w-6 text-textColor-secondary" />
                  ) : (
                    <>{/* Impossible */}</>
                  )}
                  <div>
                    <p className="text-sm font-medium">{voter.user.email}</p>
                    <p className="text-sm text-textColor-secondary">
                      {voter.user.roles.map(formatRole).join(", ")}
                    </p>
                    {
                      <p className="text-xs text-textColor-secondary">
                        {voter.didApprove && "Approved"}
                        {voter.didDeny && "Denied"}
                        {!voter.didVote && "Has not voted yet"}
                      </p>
                    }
                  </div>
                </div>
              ))}
          </CardContent>
        </Card>
      )}
    </>
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
  disbursals,
  balance,
}: {
  status: CreditFacilityStatus
  facilityAmount: number
  disbursals: { status: DisbursalStatus }[]
  balance: { outstanding: { usdBalance: number } }
}) => {
  if (
    [
      CreditFacilityStatus.PendingCollateralization,
      CreditFacilityStatus.PendingApproval,
    ].includes(status)
  ) {
    return facilityAmount
  }

  if (status === CreditFacilityStatus.Active) {
    const hasApprovedDisbursals = disbursals.some(
      (d) => d.status === DisbursalStatus.Approved,
    )
    return hasApprovedDisbursals ? balance.outstanding.usdBalance : facilityAmount
  }

  return 0
}
