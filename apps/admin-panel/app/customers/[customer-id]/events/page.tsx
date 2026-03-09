"use client"

import { use } from "react"

import { CustomerEventHistory } from "../event-history"

export default function CustomerEventsPage({
  params,
}: {
  params: Promise<{ "customer-id": string }>
}) {
  const { "customer-id": customerId } = use(params)

  return <CustomerEventHistory customerId={customerId} />
}
