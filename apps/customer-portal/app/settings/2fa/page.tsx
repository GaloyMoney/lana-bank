"use client"
import React from "react"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/primitive/card"
import { SetupAuthenticator } from "@/components/settings/authenticator"
import { SetupPasskey } from "@/components/settings/passkey"

const SettingsPage = () => {
  return (
    <main className="md:max-w-[75rem] m-auto w-[90%]">
      <Card className="mt-24">
        <CardHeader>
          <CardTitle>Setup Two Factor Authentication</CardTitle>
          <CardDescription>Choose a method for securing your account.</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-row justify-between">
          <SetupAuthenticator />
          <SetupPasskey />
        </CardContent>
      </Card>
    </main>
  )
}

export default SettingsPage
