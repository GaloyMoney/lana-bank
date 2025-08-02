"use client"

import { useEffect } from "react"
import { useRouter } from "next/navigation"

const VerifyPage: React.FC = () => {
  const router = useRouter()

  useEffect(() => {
    router.replace("/auth/login")
  }, [router])

  return (
    <div className="flex items-center justify-center">
      <div className="text-md">Redirecting to login...</div>
    </div>
  )
}

export default VerifyPage
