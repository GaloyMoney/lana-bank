"use client"
import React from "react"
import { gql } from "@apollo/client"

import { PolicyDetailsCard } from "./details"

import { PageHeading } from "@/components/page-heading"
import { useGetPolicyDetailsQuery } from "@/lib/graphql/generated"
import { CommitteeUsers } from "@/app/committees/[committee-id]/users"

gql`
  query GetPolicyDetails($id: UUID!) {
    policy(id: $id) {
      id
      policyId
      approvalProcessType
      rules {
        ... on CommitteeThreshold {
          threshold
          committee {
            id
            committeeId
            createdAt
            name
            currentMembers {
              userId
              email
              roles
            }
          }
        }
        ... on SystemApproval {
          autoApprove
        }
      }
    }
  }
`

function PolicyPage({
  params,
}: {
  params: {
    "policy-id": string
  }
}) {
  const { "policy-id": policyId } = params
  const { data, loading, error } = useGetPolicyDetailsQuery({
    variables: { id: policyId },
  })

  if (loading) return <p>Loading...</p>
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.policy) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto">
      <PageHeading>Policy Details</PageHeading>
      <div className="flex flex-col gap-5">
        <PolicyDetailsCard policy={data.policy} />
        {data.policy.rules.__typename === "CommitteeThreshold" && (
          <CommitteeUsers showRemove={false} committee={data.policy.rules.committee} />
        )}
      </div>
    </main>
  )
}

export default PolicyPage
