"use client"

import React, { useEffect, use } from "react"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { CommitteeDetailsCard } from "./details"

import { CommitteeUsers } from "./users"

import { useGetCommitteeDetailsQuery } from "@/lib/graphql/generated"
import { useBreadcrumb } from "@/app/breadcrumb-provider"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useCreateContext } from "@/app/create"

gql`
  query GetCommitteeDetails($id: UUID!) {
    committee(committeeId: $id) {
      ...CommitteeFields
    }
  }
`

function CommitteePage({
  params,
}: {
  params: Promise<{
    "committee-id": string
  }>
}) {
  const { "committee-id": committeeId } = use(params)
  const { setCustomLinks, resetToDefault } = useBreadcrumb()
  const { setCommittee } = useCreateContext()
  const navTranslations = useTranslations("Sidebar.navItems")

  const { data, loading, error } = useGetCommitteeDetailsQuery({
    variables: { id: committeeId },
  })

  useEffect(() => {
    if (data?.committee) {
      setCustomLinks([
        { title: navTranslations("committees"), href: "/committees" },
        { title: data.committee.name, isCurrentPage: true },
      ])
    }

    return () => {
      resetToDefault()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data?.committee])

  useEffect(() => {
    data?.committee && setCommittee(data?.committee)
    return () => setCommittee(null)
  }, [data?.committee, setCommittee])

  if (loading && !data) {
    return <DetailsPageSkeleton tabs={0} detailItems={3} tabsCards={1} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.committee) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto">
      <CommitteeDetailsCard committee={data.committee} />
      <div className="mt-2">
        <CommitteeUsers committee={data.committee} />
      </div>
    </main>
  )
}

export default CommitteePage
