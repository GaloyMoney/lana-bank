import React from "react"
import { SettingsFlow } from "@ory/client"

import { cookies } from "next/headers"

import { AxiosError } from "axios"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import { Separator } from "@lana/web/ui/separator"

import { SetupAuthenticator } from "@/components/settings/authenticator"
import { SetupWebAuth } from "@/components/settings/webauth"
import { kratosPublic } from "@/lib/kratos/sdk"

const SettingsPage = async ({
  searchParams: { onboard },
}: {
  searchParams?: { onboard: string }
}) => {
  let settingsFlowResponse: SettingsFlow
  const cookieParam = cookies()
    .getAll()
    .reduce((acc, cookie) => `${acc}${cookie.name}=${cookie.value}; `, "")

  try {
    settingsFlowResponse = (
      await kratosPublic().createBrowserSettingsFlow({
        cookie: cookieParam,
      })
    ).data
  } catch (error) {
    let errorMessage = "An unexpected error occurred."
    if (error instanceof AxiosError) {
      if (error.response?.data?.ui?.messages[0]?.text) {
        errorMessage = error.response.data.ui.messages[0].text
      }
    }
    return (
      <main className="md:max-w-[70rem] m-auto w-[90%]">
        <Card className="mt-10">
          <CardHeader>
            <CardTitle>Error</CardTitle>
          </CardHeader>
          <CardContent className="flex gap-4">{errorMessage}</CardContent>
        </Card>
      </main>
    )
  }

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

  return (
    <main className="md:max-w-[70rem] m-auto w-[90%]">
      {onboard && (
        <div className="bg-secondary p-2">
          Please set up two-factor authentication to continue
        </div>
      )}
      <Card className="mt-10">
        <CardHeader>
          <CardTitle>Setup Two Factor Authentication</CardTitle>
          <CardDescription>Choose a method for securing your account.</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          <SetupAuthenticator totpUnlinkNode={totpUnlinkNode} />
          <Separator />
          <SetupWebAuth addedWebAuthNode={addedWebAuthNode} />
        </CardContent>
      </Card>
    </main>
  )
}

export default SettingsPage
