"use client"

import { use } from "react"

import { CreditFacilityEventHistory } from "../event-history"

export default function CreditFacilityEventsPage({
  params,
}: {
  params: Promise<{ "credit-facility-id": string }>
}) {
  const { "credit-facility-id": publicId } = use(params)

  return <CreditFacilityEventHistory publicId={publicId} />
}
