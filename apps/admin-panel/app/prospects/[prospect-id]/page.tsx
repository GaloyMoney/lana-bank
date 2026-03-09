"use client"

import { use } from "react"

import { ProspectEventHistory } from "./event-history"

export default function ProspectPage({
  params,
}: {
  params: Promise<{ "prospect-id": string }>
}) {
  const { "prospect-id": prospectId } = use(params)

  return <ProspectEventHistory prospectId={prospectId} />
}
