import React from "react"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import { DetailItem, DetailsGroup } from "@/components/details"
import Balance from "@/components/balance/balance"

import {
  GetLoanDetailsQuery,
  useGetRealtimePriceUpdatesQuery,
} from "@/lib/graphql/generated"
import { currencyConverter } from "@/lib/utils"

type LoanSnapshotProps = {
  loan: NonNullable<GetLoanDetailsQuery["loan"]>
}

export const LoanSnapshot: React.FC<LoanSnapshotProps> = ({ loan }) => {
  const { data: priceInfo } = useGetRealtimePriceUpdatesQuery({
    fetchPolicy: "cache-only",
  })

  const MarginCallPrice = calculatePrice({
    cvlPercentage: loan?.loanTerms.marginCallCvl,
    principalInCents: loan?.facility,
    collateralInSatoshis: loan?.balance.collateral.btcBalance,
  })

  const LiquidationCallPrice = calculatePrice({
    cvlPercentage: loan?.loanTerms.liquidationCvl,
    principalInCents: loan?.facility,
    collateralInSatoshis: loan?.balance.collateral.btcBalance,
  })

  return (
    <Card className="mt-4">
      <CardHeader>
        <CardTitle>Loan Snapshot</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-2 gap-6">
          <div className="grid auto-rows-min">
            <DetailsGroup>
              <DetailItem
                label="Collateral balance"
                valueComponent={
                  <Balance amount={loan.balance.collateral.btcBalance} currency="btc" />
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
                value={`${loan.currentCvl}%`}
              />
              <DetailItem
                label={`Collateral to reach target (${loan.loanTerms.initialCvl}%)`}
                valueComponent={
                  <Balance amount={loan.collateralToMatchInitialCvl} currency="btc" />
                }
              />
              {loan.balance.collateral.btcBalance > 0 ? (
                <DetailItem
                  label={`Margin Call Price BTC/USD (${loan.loanTerms.marginCallCvl}%)`}
                  valueComponent={<Balance amount={MarginCallPrice} currency="usd" />}
                />
              ) : (
                <DetailItem
                  label="Margin Call CVL"
                  value={`${loan.loanTerms.marginCallCvl}%`}
                />
              )}
              {loan.balance.collateral.btcBalance > 0 ? (
                <DetailItem
                  label={`Liquidation Call Price BTC/USD (${loan.loanTerms.liquidationCvl}%)`}
                  valueComponent={
                    <Balance amount={LiquidationCallPrice} currency="usd" />
                  }
                />
              ) : (
                <DetailItem
                  label="Liquidation Call CVL"
                  value={`${loan.loanTerms.liquidationCvl}%`}
                />
              )}
            </DetailsGroup>
          </div>
          <div className="grid auto-rows-min">
            <DetailsGroup>
              <DetailItem
                label="Outstanding Balance"
                valueComponent={
                  <Balance amount={loan.balance.outstanding.usdBalance} currency="usd" />
                }
              />
              <DetailItem
                label="Interest incurred"
                valueComponent={
                  <Balance
                    amount={loan.balance.interestIncurred.usdBalance}
                    currency="usd"
                  />
                }
              />
            </DetailsGroup>
          </div>
        </div>
      </CardContent>
    </Card>
  )
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
