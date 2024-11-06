"use server"

import axios from "axios"

import { env } from "@/env"
// to send magic link after a user is created by super-user.
export async function sendMagicLinkToEmail(email: string) {
  try {
    const csrfTokenResponse = await axios.get(`${env.NEXTAUTH_URL}/csrf`, {
      withCredentials: true,
    })
    const csrfTokenCookie = csrfTokenResponse.headers["set-cookie"] || []
    const { csrfToken } = csrfTokenResponse.data

    const signInResponse = await axios.post(
      `${env.NEXTAUTH_URL}/signin/email`,
      {
        email,
        csrfToken,
        callbackUrl:
          process.env.NODE_ENV === "development" ? "/admin-panel/profile" : "/profile",
        json: true,
      },
      {
        headers: {
          "Content-Type": "application/json",
          "Cookie": csrfTokenCookie.join("; "),
        },
      },
    )

    if (signInResponse.status === 200) {
      console.log("Email sign-in initiated successfully")
      return { success: true, message: "Check your email for the login link" }
    } else {
      console.error("Failed to initiate email sign-in:", signInResponse.data)
      return { success: false, message: "Failed to send login email" }
    }
  } catch (error) {
    console.error("Error in sendMagicLinkToEmail:", error)
    return { success: false, message: "An unexpected error occurred" }
  }
}
