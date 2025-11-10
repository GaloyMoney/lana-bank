import { DetailsCard } from "@lana/web/components/details"

import React from "react"

import Balance from "@/components/balance"
import { GetCreditFacilityQuery } from "@/lib/graphql/generated"

type CreditFacilityData = NonNullable<GetCreditFacilityQuery["creditFacility"]>

function FacilityCard({ data }: { data: CreditFacilityData }) {
  const facilityData = [
    {
      label: "Facility Amount",
      value: <Balance amount={data.facilityAmount} currency="usd" />,
    },
    {
      label: "Facility Remaining",
      value: (
        <Balance amount={data.balance.facilityRemaining.usdBalance} currency="usd" />
      ),
    },
    {
      label: "Outstanding Interest",
      value: (
        <Balance amount={data.balance.interest.outstanding.usdBalance} currency="usd" />
      ),
    },
    {
      label: "Total Outstanding Balance",
      value: <Balance amount={data.balance.outstanding.usdBalance} currency="usd" />,
    },
  ]
  return (
    <DetailsCard className="w-full" title="Facility" details={facilityData} columns={2} />
  )
}

export default FacilityCard
