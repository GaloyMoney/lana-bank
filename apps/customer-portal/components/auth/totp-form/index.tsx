"use client"
import { useState } from "react"

import { Button } from "@lana/web/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"
import { Input } from "@lana/web/ui/input"
import { Alert, AlertDescription } from "@lana/web/ui/alert"

import { useRouter } from "next/navigation"

import { submitTotpFow } from "@/lib/kratos/public/submit-totp"

const TotpForm = ({ flowId }: { flowId: string }) => {
  const router = useRouter()
  const [totpCode, setTotpCode] = useState("")
  const [error, setError] = useState<string | null>(null)

  const handleTotpSubmit = async (event: React.FormEvent) => {
    event.preventDefault()
    setError(null)
    if (!totpCode) {
      return
    }

    const submitTotpFowResponse = await submitTotpFow({
      flowId,
      totpCode,
    })

    if (submitTotpFowResponse instanceof Error) {
      setError(submitTotpFowResponse.message)
      return
    }

    router.replace("/")
    setTimeout(() => window.location.reload(), 1) // to ensure the layout is loaded
  }

  return (
    <Card className="md:w-2/5">
      <CardHeader className="pt-4">
        <CardTitle>Authenticator Code</CardTitle>
        <CardDescription className="text-textColor-secondary">
          Please enter your authenticator code to continue.
        </CardDescription>
      </CardHeader>
      <form onSubmit={handleTotpSubmit}>
        <CardContent>
          <Input
            type="text"
            value={totpCode}
            onChange={(e) => setTotpCode(e.target.value)}
            placeholder="Please enter code"
            data-test-id="auth-totp-input"
          />
        </CardContent>
        <CardFooter className="flex flex-col gap-2">
          <Button
            data-test-id="auth-totp-submit-btn"
            type="submit"
            className="rounded-full px-6 w-full"
          >
            Next
          </Button>
          {error && (
            <Alert variant="destructive" className="mt-1 p-3">
              <AlertDescription data-test-id="auth-totp-error">{error}</AlertDescription>
            </Alert>
          )}
        </CardFooter>
      </form>
    </Card>
  )
}

export { TotpForm }
