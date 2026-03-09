"use client"
import React, { useEffect, use } from "react"
import { gql } from "@apollo/client"

import { useTranslations } from "next-intl"

import { Tabs, TabsList, TabsTrigger, TabsContent } from "@lana/web/ui/tabs"

import { PolicyDetailsCard } from "./details"
import { PolicyEventHistory } from "./event-history"

import { NotFound } from "@/components/not-found"


import { useGetPolicyDetailsQuery } from "@/lib/graphql/generated"
import { CommitteeUsers } from "@/app/committees/[committee-id]/users"
import { useBreadcrumb } from "@/app/breadcrumb-provider"
import { useProcessTypeLabel } from "@/app/actions/hooks"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useCreateContext } from "@/app/create"

gql`
  query GetPolicyDetails($id: UUID!) {
    policy(id: $id) {
      id
      policyId
      approvalProcessType
      rules {
        ... on CommitteeApproval {
          committee {
            ...CommitteeFields
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
  params: Promise<{
    "policy-id": string
  }>
}) {
  const { "policy-id": policyId } = use(params)
  const { setCustomLinks, resetToDefault } = useBreadcrumb()
  const { setPolicy } = useCreateContext()
  const navTranslations = useTranslations("Sidebar.navItems")
  const tTabs = useTranslations("Policies.PolicyDetails.tabs")

  const processTypeLabel = useProcessTypeLabel()

  const { data, loading, error } = useGetPolicyDetailsQuery({
    variables: { id: policyId },
  })

  useEffect(() => {
    if (data?.policy) {
      setCustomLinks([
        { title: navTranslations("policies"), href: "/policies" },
        {
          title: processTypeLabel(data.policy.approvalProcessType),
          isCurrentPage: true,
        },
      ])
    }

    return () => {
      resetToDefault()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data?.policy])

  useEffect(() => {
    data?.policy && setPolicy(data?.policy)
    return () => setPolicy(null)
  }, [data?.policy, setPolicy])

  if (loading && !data) {
    return <DetailsPageSkeleton tabs={0} detailItems={3} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.policy) return <NotFound />

  return (
    <main className="max-w-7xl m-auto space-y-2">
      <PolicyDetailsCard policy={data.policy} />
      <Tabs defaultValue={data.policy.rules.__typename === "CommitteeApproval" ? "members" : "events"}>
        <TabsList>
          {data.policy.rules.__typename === "CommitteeApproval" && (
            <TabsTrigger value="members">{tTabs("members")}</TabsTrigger>
          )}
          <TabsTrigger value="events">{tTabs("events")}</TabsTrigger>
        </TabsList>
        {data.policy.rules.__typename === "CommitteeApproval" && (
          <TabsContent value="members">
            <CommitteeUsers showRemove={false} committee={data.policy.rules.committee} />
          </TabsContent>
        )}
        <TabsContent value="events">
          <PolicyEventHistory policyId={policyId} />
        </TabsContent>
      </Tabs>
    </main>
  )
}

export default PolicyPage
