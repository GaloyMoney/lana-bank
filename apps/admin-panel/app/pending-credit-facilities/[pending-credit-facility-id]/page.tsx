"use client"

import { use } from "react"
import { gql } from "@apollo/client"

import { useGetPendingCreditFacilityRepaymentPlanQuery } from "@/lib/graphql/generated"
import { CreditFacilityRepaymentPlan } from "@/app/credit-facilities/[credit-facility-id]/repayment-plan/list"

interface PendingCreditFacilityDetailsPageProps {
  params: Promise<{
    "pending-credit-facility-id": string
  }>
}

gql`
  query GetPendingCreditFacilityRepaymentPlan($id: UUID!) {
    pendingCreditFacility(id: $id) {
      id
      pendingCreditFacilityId
      repaymentPlan {
        ...RepaymentOnFacilityPage
      }
    }
  }
`

export default function PendingCreditFacilityDetailsPage({
  params,
}: PendingCreditFacilityDetailsPageProps) {
  const { "pending-credit-facility-id": pendingId } = use(params)
  const { data } = useGetPendingCreditFacilityRepaymentPlanQuery({
    variables: { id: pendingId },
  })

  if (!data?.pendingCreditFacility) return null

  return <CreditFacilityRepaymentPlan creditFacility={data.pendingCreditFacility} />
}
