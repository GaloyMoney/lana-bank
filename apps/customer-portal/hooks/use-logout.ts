import { useState } from "react"

import { toast } from "sonner"

import { kratosPublic } from "@/lib/kratos/sdk"

const useLogout = () => {
  const [loading, setLoading] = useState(false)

  const logout = async () => {
    setLoading(true)
    try {
      const { data } = await kratosPublic().createBrowserLogoutFlow()
      await kratosPublic().updateLogoutFlow({ token: data.logout_token })
      window.location.href = "/auth"
    } catch (error) {
      setLoading(false)
      if (error instanceof Error) toast(error.message)
      else toast("An error occurred while logging out")
    }

    setLoading(false)
  }

  return {
    loading,
    logout,
  }
}

export { useLogout }
