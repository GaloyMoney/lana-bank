import { redirect } from "next/navigation"

import { AuthTemplateCard } from "@/components/auth/auth-template-card"
import { OtpForm } from "@/components/auth/otp-form"

export type OtpParams = {
  flowId?: string
  type?: string
}

async function Otp({ searchParams }: { searchParams: OtpParams }) {
  const flowId = searchParams?.flowId

  if (!flowId) {
    redirect("/auth")
  }

  const email = "user@example.com" // TODO

  return (
    <AuthTemplateCard>
      <OtpForm email={email} flowId={flowId} />
    </AuthTemplateCard>
  )
}

export default Otp
