"use client"

import { useRef, useState } from "react"
import { useRouter } from "next/navigation"
import { z } from "zod"

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

import { createAuthFlow } from "@/lib/kratos/public/create-auth-flow"

const emailSchema = z.string().email({ message: "Invalid email address" })

const AuthForm = () => {
  const router = useRouter()

  const emailRef = useRef<HTMLInputElement>(null)
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault()
    if (emailRef.current && emailRef.current.value) {
      setError(null)
      const result = emailSchema.safeParse(emailRef.current.value)
      if (!result.success) {
        setError(result.error.issues[0].message)
        return
      }

      try {
        const { flowId, type } = await createAuthFlow({ email: emailRef.current.value })
        router.push(`/auth/otp?flowId=${flowId}&type=${type}`)
      } catch (e) {
        console.error(e)
        setError("Something went wrong. Please try again.")
      }
    }
  }

  return (
    <Card className="md:w-2/5 my-2" variant="transparent">
      <CardHeader>
        <CardTitle>Welcome</CardTitle>
        <CardDescription>
          Getting started is easy. Simply enter and confirm your email address.
        </CardDescription>
      </CardHeader>
      <form onSubmit={handleSubmit}>
        <CardContent>
          <Input
            data-test-id="auth-email-input"
            type="email"
            ref={emailRef}
            placeholder="Please enter email"
          />
        </CardContent>
        <CardFooter className="flex flex-col gap-2">
          <Button data-test-id="auth-email-submit-btn" type="submit" className="w-full">
            Next
          </Button>
          {error && (
            <Alert variant="destructive" className="mt-1 p-3">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
        </CardFooter>
      </form>
    </Card>
  )
}

export { AuthForm }
