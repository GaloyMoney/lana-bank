"use client"

import { useState, useEffect } from "react"
import { useRouter, useSearchParams } from "next/navigation"

import { Button } from "@lana/web/ui/button"

import { loginUser } from "../ory"

import { Input } from "@/components/input"

const Login: React.FC = () => {
  const router = useRouter()
  const searchParams = useSearchParams()

  const [isLoading, setIsLoading] = useState(false)
  const [email, setEmail] = useState("")
  const [error, setError] = useState("")
  const [timeoutMessage, setTimeoutMessage] = useState("")

  useEffect(() => {
    const reason = searchParams.get("reason")
    if (reason === "session_timeout") {
      setTimeoutMessage(
        "Your session has expired due to inactivity. Please log in again.",
      )
    }
  }, [searchParams])

  const onSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    setIsLoading(true)
    setError("")

    try {
      const flowId = await loginUser(email)
      router.push(`/auth/verify?flow=${flowId}`)
    } catch {
      setError("Please check your credentials and try again.")
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <>
      <h1 className="font-semibold leading-none tracking-tight text-xl">Sign In</h1>
      <div className="space-y-[10px]">
        <div className="text-md">Welcome to Lana Bank Admin Panel</div>
        <div className="text-md font-light">Enter your email address to continue</div>
      </div>
      {timeoutMessage && (
        <div className="p-3 bg-amber-50 border border-amber-200 rounded-md text-amber-800">
          {timeoutMessage}
        </div>
      )}
      <form className="space-y-[20px] w-full" onSubmit={onSubmit}>
        <Input
          label="Your email"
          type="email"
          name="email"
          autofocus
          placeholder="Please enter your email address"
          defaultValue={email}
          onChange={setEmail}
        />
        <Button type="submit" loading={isLoading}>
          Submit
        </Button>
        {error && <div className="text-destructive">{error}</div>}
      </form>
    </>
  )
}

export default Login
