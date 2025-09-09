"use client"

import { gql } from "@apollo/client"
import { use } from "react"

import { useGetCreditFacilityProposalRepaymentPlanQuery } from "@/lib/graphql/generated"
import { CreditFacilityRepaymentPlan } from "@/app/credit-facilities/[credit-facility-id]/repayment-plan/list"

gql`
  query GetCreditFacilityProposalRepaymentPlan($id: UUID!) {
    creditFacilityProposal(id: $id) {
      id
      creditFacilityProposalId
      repaymentPlan {
        ...RepaymentOnFacilityPage
      }
    }
  }
`

export default function CreditFacilityProposalRepaymentPlansPage({
  params,
}: {
  params: Promise<{ "credit-facility-proposal-id": string }>
}) {
  const { "credit-facility-proposal-id": proposalId } = use(params)
  const { data } = useGetCreditFacilityProposalRepaymentPlanQuery({
    variables: { id: proposalId },
  })

  if (!data?.creditFacilityProposal) return null

  return <CreditFacilityRepaymentPlan creditFacility={data.creditFacilityProposal} />
}
