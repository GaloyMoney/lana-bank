"use client"

import { useRef, useState } from "react"
import { useRouter } from "next/navigation"
import { z } from "zod"

import { Button } from "@/components/primitive/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/primitive/card"
import { Input } from "@/components/primitive/input"
import { Alert, AlertDescription } from "@/components/primitive/alert"
import { createAuthFlow } from "@/lib/kratos/public/auth-flow"

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
        setError(result.error.errors[0].message)
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
    <Card variant="transparent" className="md:w-2/5">
      <CardHeader className="pt-4">
        <CardTitle>Create an Account / Sign-in</CardTitle>
        <CardDescription className="text-textColor-secondary">
          Getting started is easy. Simply enter and confirm your email address.
        </CardDescription>
      </CardHeader>
      <form onSubmit={handleSubmit}>
        <CardContent>
          <Input type="email" ref={emailRef} placeholder="Please enter email" />
        </CardContent>
        <CardFooter className="flex flex-col gap-2">
          <Button type="submit" className="rounded-full px-6 w-full">
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
