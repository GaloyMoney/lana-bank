import React from "react"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import { DetailItem, DetailsGroup } from "@/components/details"

import { GetCreditFacilityDetailsQuery } from "@/lib/graphql/generated"
import { formatDate, formatInterval, formatPeriod } from "@/lib/utils"
import Balance from "@/components/balance/balance"

type CreditFacilityTermsProps = {
  creditFacility: NonNullable<GetCreditFacilityDetailsQuery["creditFacility"]>
}

export const CreditFacilityTerms: React.FC<CreditFacilityTermsProps> = ({
  creditFacility,
}) => {
  return (
    <Card className="mt-4">
      <CardHeader>
        <CardTitle>Credit Facility Terms</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-3 gap-6">
          <div className="grid auto-rows-min">
            <DetailsGroup>
              <DetailItem
                label="Duration"
                value={`${creditFacility.creditFacilityTerms.duration.units} ${formatPeriod(creditFacility.creditFacilityTerms.duration.period)}`}
              />
              <DetailItem
                label="Interest (APR)"
                value={`${creditFacility.creditFacilityTerms.annualRate}%`}
              />
              <DetailItem
                label="Accrual Interval"
                value={formatInterval(creditFacility.creditFacilityTerms.accrualInterval)}
              />
            </DetailsGroup>
          </div>
          <div className="grid auto-rows-min">
            <DetailsGroup>
              <DetailItem
                label="Target/initial CVL %"
                value={`${creditFacility.creditFacilityTerms.initialCvl}%`}
              />
              <DetailItem
                label="Margin call CVL %"
                value={`${creditFacility.creditFacilityTerms.marginCallCvl}%`}
              />
              <DetailItem
                label="Liquidation CVL %"
                value={`${creditFacility.creditFacilityTerms.liquidationCvl}%`}
              />
            </DetailsGroup>
          </div>
          <div className="grid auto-rows-min">
            <DetailsGroup>
              <DetailItem
                label="Date created"
                value={formatDate(creditFacility.createdAt)}
              />
              <DetailItem
                label="Facility Amount"
                value={<Balance amount={creditFacility.facilityAmount} currency="usd" />}
              />
              <DetailItem
                label="Incurrence Interval"
                value={formatInterval(
                  creditFacility.creditFacilityTerms.incurrenceInterval,
                )}
              />
            </DetailsGroup>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
