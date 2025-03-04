"use client"

import { useEffect, useState } from "react"
import { useRouter } from "next/navigation"

import { Button } from "@lana/web/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"
import { InputOTP, InputOTPGroup, InputOTPSlot } from "@lana/web/ui/input-otp"

import { Alert, AlertDescription } from "@lana/web/ui/alert"

import { submitAuthFlow } from "@/lib/kratos/public/submit-auth-flow"

export type OtpParams = {
  flowId: string
  type: "login" | "register"
}

const OtpForm: React.FC<OtpParams> = ({ flowId, type }) => {
  const router = useRouter()
  const [otp, setOtp] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  const handleOtpSubmission = async () => {
    if (otp.length !== 6) {
      return
    }
    setError(null)
    setLoading(true)
    try {
      await submitAuthFlow({ flowId, otp, type })
      router.replace("/auth/2fa")
    } catch (error) {
      console.error(error)
      setError("Invalid OTP or OTP has expired. Please go back and try again.")
    } finally {
      setLoading(false)
    }
  }

  const submitOtpHandler = async (
    event: React.MouseEvent<HTMLButtonElement, MouseEvent>,
  ) => {
    event.preventDefault()
    handleOtpSubmission()
  }

  useEffect(() => {
    if (otp.length === 6) {
      handleOtpSubmission()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [otp, flowId, type, router])

  return (
    <Card className="md:w-2/5" variant="transparent">
      <CardHeader className="pt-4">
        <CardTitle>One time Password</CardTitle>
        <CardDescription>
          An email has been sent to your email address. Please enter the OTP to continue.
        </CardDescription>
      </CardHeader>
      <form>
        <CardContent>
          <InputOTP
            data-test-id="auth-otp-input"
            value={otp}
            onChange={(value) => setOtp(value)}
            maxLength={6}
          >
            <InputOTPGroup className="w-full">
              <InputOTPSlot className="w-1/6 h-12" index={0} />
              <InputOTPSlot className="w-1/6 h-12" index={1} />
              <InputOTPSlot className="w-1/6 h-12" index={2} />
              <InputOTPSlot className="w-1/6 h-12" index={3} />
              <InputOTPSlot className="w-1/6 h-12" index={4} />
              <InputOTPSlot className="w-1/6 h-12" index={5} />
            </InputOTPGroup>
          </InputOTP>
        </CardContent>
        <CardFooter className="flex flex-col gap-2">
          <Button
            data-test-id="auth-otp-submit-btn"
            type="submit"
            className="w-full"
            onClick={submitOtpHandler}
            disabled={loading}
          >
            Submit
          </Button>
          {error && (
            <Alert variant="destructive" className="mt-1 p-3">
              <AlertDescription data-test-id="auth-otp-error">{error}</AlertDescription>
            </Alert>
          )}
        </CardFooter>
      </form>
    </Card>
  )
}

export { OtpForm }
