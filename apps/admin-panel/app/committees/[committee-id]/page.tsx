"use client"

import React, { useEffect, use } from "react"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { Tabs, TabsList, TabsTrigger, TabsContent } from "@lana/web/ui/tabs"

import { CommitteeDetailsCard } from "./details"
import { CommitteeEventHistory } from "./event-history"

import { CommitteeUsers } from "./users"

import { NotFound } from "@/components/not-found"

import { useGetCommitteeDetailsQuery } from "@/lib/graphql/generated"
import { useBreadcrumb } from "@/app/breadcrumb-provider"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useCreateContext } from "@/app/create"

gql`
  query GetCommitteeDetails($id: UUID!) {
    committee(id: $id) {
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
  const tTabs = useTranslations("Committees.CommitteeDetails.tabs")

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
  if (!data?.committee) return <NotFound />

  return (
    <main className="max-w-7xl m-auto space-y-2">
      <CommitteeDetailsCard committee={data.committee} />
      <Tabs defaultValue="members">
        <TabsList>
          <TabsTrigger value="members">{tTabs("members")}</TabsTrigger>
          <TabsTrigger value="events">{tTabs("events")}</TabsTrigger>
        </TabsList>
        <TabsContent value="members">
          <CommitteeUsers committee={data.committee} />
        </TabsContent>
        <TabsContent value="events">
          <CommitteeEventHistory committeeId={committeeId} />
        </TabsContent>
      </Tabs>
    </main>
  )
}

export default CommitteePage
