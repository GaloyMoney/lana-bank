import { useState } from "react"
import { signOut } from "next-auth/react"

import { toast } from "sonner"

import { env } from "@/env"

const useLogout = () => {
  const [loading, setLoading] = useState(false)

  const logout = async () => {
    setLoading(true)
    try {
      await signOut({ redirect: false })
      const keycloakLogoutUrl = new URL(
        `/realms/${env.NEXT_PUBLIC_KEYCLOAK_REALM}/protocol/openid-connect/logout`,
        env.NEXT_PUBLIC_KEYCLOAK_URL,
      )
      keycloakLogoutUrl.searchParams.set("client_id", env.NEXT_PUBLIC_KEYCLOAK_CLIENT_ID)
      keycloakLogoutUrl.searchParams.set(
        "post_logout_redirect_uri",
        window.location.origin + "/",
      )
      window.location.href = keycloakLogoutUrl.toString()
    } catch (error) {
      setLoading(false)
      if (error instanceof Error) toast(error.message)
      else toast("An error occurred while logging out")
    }
  }

  return {
    loading,
    logout,
  }
}

export { useLogout }
