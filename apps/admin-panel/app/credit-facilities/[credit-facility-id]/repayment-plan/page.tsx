"use client"

import { gql } from "@apollo/client"
import { use } from "react"

import { CreditFacilityRepaymentPlan } from "./list"

import { useGetCreditFacilityRepaymentPlanQuery } from "@/lib/graphql/generated"

gql`
  fragment RepaymentOnFacilityPage on CreditFacilityRepaymentPlanEntry {
    repaymentType
    status
    initial
    outstanding
    accrualAt
    dueAt
  }

  query GetCreditFacilityRepaymentPlan($publicId: PublicId!) {
    creditFacilityByPublicId(id: $publicId) {
      id
      creditFacilityId
      repaymentPlan {
        ...RepaymentOnFacilityPage
      }
    }
  }
`
export default function CreditFacilityRepaymentPlansPage({
  params,
}: {
  params: Promise<{ "credit-facility-id": string }>
}) {
  const { "credit-facility-id": publicId } = use(params)
  const { data } = useGetCreditFacilityRepaymentPlanQuery({
    variables: { publicId },
  })

  if (!data?.creditFacilityByPublicId) return null

  return <CreditFacilityRepaymentPlan creditFacility={data.creditFacilityByPublicId} />
}
