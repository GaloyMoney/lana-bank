"use client"

import { useEffect, useState } from "react"

import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import { Button } from "@lana/web/ui/button"
import { Alert, AlertDescription } from "@lana/web/ui/alert"

import { useRouter } from "next/navigation"

import {
  SerializedPublicKeyCredentialRequestOptions,
  signinWithPasskey,
} from "@/lib/webauth"
import { submitWebAuthnFow } from "@/lib/kratos/public/submit-webauthn"

const PasskeySignIn = ({
  flowId,
  publicKey,
  email,
  csrfToken,
}: {
  flowId: string
  publicKey: SerializedPublicKeyCredentialRequestOptions
  email: string
  csrfToken: string
}) => {
  const router = useRouter()
  const [error, setError] = useState<string | null>(null)

  const startWebAuth = async () => {
    const signinWithPasskeyResponse = await signinWithPasskey(publicKey)
    const submitWebAuthResponse = await submitWebAuthnFow({
      flowId,
      webAuthLogin: signinWithPasskeyResponse,
      email,
      csrfToken,
    })

    if (submitWebAuthResponse instanceof Error) {
      setError(submitWebAuthResponse.message)
      return
    }

    router.replace("/")
    setTimeout(() => window.location.reload(), 1) // to ensure the layout is loaded
  }

  useEffect(() => {
    startWebAuth()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  return (
    <Card className="md:w-2/5">
      <CardHeader className="pt-4">
        <CardTitle>Continue With Passkey</CardTitle>
        <CardDescription className="text-textColor-secondary">
          Connect your Passkey to sign in.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Button
          onClick={startWebAuth}
          className="align-middle w-30 items-center min-h-max"
        >
          Sign in with Passkey
        </Button>
      </CardContent>

      <CardFooter className="flex flex-col gap-2">
        {error && (
          <Alert variant="destructive" className="mt-1 p-3">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}
      </CardFooter>
    </Card>
  )
}

export { PasskeySignIn }
