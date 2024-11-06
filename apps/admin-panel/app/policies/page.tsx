"use client"
import React from "react"
import { gql } from "@apollo/client"
import { useRouter } from "next/navigation"

import { PageHeading } from "@/components/page-heading"
import { Card, CardContent } from "@/components/primitive/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import { usePoliciesQuery } from "@/lib/graphql/generated"
import { formatProcessType } from "@/lib/utils"

gql`
  query Policies($first: Int!, $after: String) {
    policies(first: $first, after: $after) {
      edges {
        cursor
        node {
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
              }
            }
            ... on SystemApproval {
              autoApprove
            }
          }
        }
      }
      pageInfo {
        hasPreviousPage
        hasNextPage
        startCursor
        endCursor
      }
    }
  }
`

function PoliciesPage() {
  const router = useRouter()
  const { data, loading, error, fetchMore } = usePoliciesQuery({
    variables: {
      first: 20,
    },
    fetchPolicy: "cache-and-network",
  })

  if (loading && !data) {
    return (
      <main>
        <div className="flex justify-between items-center mb-8">
          <PageHeading className="mb-0">Policies</PageHeading>
        </div>
        <Card>
          <CardContent>
            <p className="mt-6">Loading...</p>
          </CardContent>
        </Card>
      </main>
    )
  }

  if (error) {
    return (
      <main>
        <div className="flex justify-between items-center mb-8">
          <PageHeading className="mb-0">Policies</PageHeading>
        </div>
        <Card>
          <CardContent>
            <p className="text-destructive mt-6">{error.message}</p>
          </CardContent>
        </Card>
      </main>
    )
  }

  return (
    <main>
      <div className="flex justify-between items-center mb-8">
        <PageHeading className="mb-0">Policies</PageHeading>
      </div>

      <Card>
        <CardContent>
          {!data?.policies.edges || data.policies.edges.length === 0 ? (
            <p className="mt-6">No policies found</p>
          ) : (
            <Table className="mt-6">
              <TableHeader>
                <TableRow>
                  <TableHead>Process Type</TableHead>
                  <TableHead>Rule</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.policies.edges.map(({ node: policy }) => (
                  <TableRow
                    key={policy.policyId}
                    onClick={() => router.push(`/policies/${policy.policyId}`)}
                    className="cursor-pointer"
                  >
                    <TableCell>{formatProcessType(policy.approvalProcessType)}</TableCell>
                    <TableCell
                      className={
                        policy.rules.__typename === "CommitteeThreshold"
                          ? ""
                          : "text-textColor-secondary"
                      }
                    >
                      {policy.rules.__typename === "CommitteeThreshold" &&
                        `${policy.rules.committee.name} Committee`}
                      {policy.rules.__typename === "SystemApproval" && "System"}
                    </TableCell>
                  </TableRow>
                ))}
                {data.policies.pageInfo.hasNextPage && (
                  <TableRow
                    onClick={() =>
                      fetchMore({
                        variables: {
                          after:
                            data.policies.edges[data.policies.edges.length - 1].cursor,
                        },
                      })
                    }
                  >
                    <TableCell colSpan={4}>
                      <div className="font-thin italic">show more...</div>
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </main>
  )
}

export default PoliciesPage
