"use client"

import { useRouter } from "next/navigation"

export default function LedgerAccount() {
  const router = useRouter()
  router.push("/chart-of-accounts")

  return <></>
}
