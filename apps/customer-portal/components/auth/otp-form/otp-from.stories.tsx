import React from "react"

import { OtpForm } from "@/components/auth/otp-form"

export default {
  title: "components/Auth/otp-form",
  component: OtpForm,
}

export const AuthOtp = () => <OtpForm type="register" flowId="flow-id" />
