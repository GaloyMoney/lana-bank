"use client"

import React from "react"
import { gql } from "@apollo/client"

import { CommitteeDetailsCard } from "./details"
import { CommitteeUsers } from "./users"

import { PageHeading } from "@/components/page-heading"
import { useGetCommitteeDetailsQuery } from "@/lib/graphql/generated"

gql`
  query GetCommitteeDetails($id: UUID!) {
    committee(id: $id) {
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
`

function CommitteePage({
  params,
}: {
  params: {
    "committee-id": string
  }
}) {
  const { "committee-id": committeeId } = params
  const { data, loading, error } = useGetCommitteeDetailsQuery({
    variables: { id: committeeId },
  })

  if (loading) return <p>Loading...</p>
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.committee) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto">
      <PageHeading>Committee Details</PageHeading>
      <div className="flex flex-col gap-5">
        <CommitteeDetailsCard committee={data.committee} />
        <CommitteeUsers committee={data.committee} />
      </div>
    </main>
  )
}

export default CommitteePage
