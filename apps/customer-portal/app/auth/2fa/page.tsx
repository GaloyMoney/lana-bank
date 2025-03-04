import { redirect } from "next/navigation"

import Link from "next/link"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"
import { Button } from "@lana/web/ui/button"

import { cookies } from "next/headers"

import { AuthTemplateCard } from "@/components/auth/auth-template-card"
import { kratosPublic } from "@/lib/kratos/sdk"
import { toSession } from "@/lib/kratos/public/to-session"

async function TwoFactorAuthPage() {
  const cookie = cookies()
    .getAll()
    .reduce((acc, cookie) => `${acc}${cookie.name}=${cookie.value}; `, "")

  const session = await toSession({ cookie: cookie })
  if (session instanceof Error) {
    redirect("/auth")
  }

  const settingsFlowResponse = (
    await kratosPublic().createBrowserSettingsFlow({
      cookie,
    })
  ).data
  const addedWebAuthNode =
    settingsFlowResponse.ui.nodes.filter(
      (node) =>
        node.group === "webauthn" &&
        "name" in node.attributes &&
        node.attributes.name === "webauthn_remove",
    ) || []
  const totpUnlinkNode =
    settingsFlowResponse.ui.nodes.filter(
      (node) =>
        node.group === "totp" &&
        "name" in node.attributes &&
        node.attributes.name === "totp_unlink",
    )[0] || null

  if (addedWebAuthNode.length === 0 && !totpUnlinkNode) {
    redirect("/settings/2fa?onboard=true")
  }

  const flowId = (
    await kratosPublic().createBrowserLoginFlow({
      aal: "aal2",
      cookie,
    })
  ).data.id

  return (
    <AuthTemplateCard>
      <Card className="md:w-2/5">
        <CardHeader className="pt-4">
          <CardTitle>Continue with two-factor authentication.</CardTitle>
          <CardDescription className="text-textColor-secondary">
            Select Method to Continue your two-factor authentication.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-2 w-full">
          {addedWebAuthNode.length > 0 && (
            <Link href={`/auth/2fa/webauth?flowId=${flowId}`}>
              <Button className="align-middle w-30 items-center w-full">
                Continue with Passkey
              </Button>
            </Link>
          )}
          {totpUnlinkNode && (
            <Link href={`/auth/2fa/totp?flowId=${flowId}`}>
              <Button className="align-middle w-30 items-center min-h-max w-full">
                Continue with Authenticator
              </Button>
            </Link>
          )}
        </CardContent>
      </Card>
    </AuthTemplateCard>
  )
}
export default TwoFactorAuthPage
