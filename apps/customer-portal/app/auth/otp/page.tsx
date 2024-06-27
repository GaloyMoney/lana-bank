import { redirect } from "next/navigation"

import { AuthTemplateCard } from "@/components/auth/auth-template-card"
import { OtpForm } from "@/components/auth/otp-form"

async function RegisterOtp({
  searchParams,
}: {
  searchParams: {
    flow?: string
  }
}) {
  const flowId = searchParams?.flow

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

export default RegisterOtp
