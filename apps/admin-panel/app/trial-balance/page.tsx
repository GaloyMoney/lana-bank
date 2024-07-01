"use client"
import React from "react"

import { PageHeading } from "@/components/page-heading"
import { useGetTrialBalanceQuery } from "@/lib/graphql/generated"

function TrialBalancePage() {
  const { loading, error, data } = useGetTrialBalanceQuery()

  return (
    <main>
      <PageHeading>Trial Balance</PageHeading>
      <div>{loading}</div>
      <div>{String(error)}</div>
      <div>{JSON.stringify(data?.trialBalance, null, 2)}</div>
    </main>
  )
}

export default TrialBalancePage
