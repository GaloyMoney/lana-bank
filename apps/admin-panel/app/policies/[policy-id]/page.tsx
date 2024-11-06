"use client"
import React from "react"
import { gql } from "@apollo/client"

import { PolicyDetailsCard } from "./details"

import { useGetPolicyDetailsQuery } from "@/lib/graphql/generated"
import { CommitteeUsers } from "@/app/committees/[committee-id]/users"
import { BreadcrumbLink, BreadCrumbWrapper } from "@/components/breadcrumb-wrapper"
import { formatProcessType } from "@/lib/utils"

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

const PolicyBreadcrumb = ({ policyName }: { policyName: string }) => {
  const links: BreadcrumbLink[] = [
    { title: "Dashboard", href: "/dashboard" },
    { title: "Policies", href: "/policies" },
    { title: policyName, isCurrentPage: true },
  ]

  return <BreadCrumbWrapper links={links} />
}

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
      <PolicyBreadcrumb policyName={formatProcessType(data.policy.approvalProcessType)} />
      <PolicyDetailsCard policy={data.policy} />
      <div className="flex flex-col mt-4">
        {data.policy.rules.__typename === "CommitteeThreshold" && (
          <CommitteeUsers showRemove={false} committee={data.policy.rules.committee} />
        )}
      </div>
    </main>
  )
}

export default PolicyPage
