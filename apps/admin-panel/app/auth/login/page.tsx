"use client"

import { Button } from "@lana/web/ui/button"

import { login } from "../keycloak"

const Login: React.FC = () => {
  const handleLogin = async () => {
    try {
      await login()
    } catch (error) {
      console.error("Login failed:", error)
    }
  }

  return (
    <>
      <h1 className="font-semibold leading-none tracking-tight text-xl">Sign In</h1>
      <div className="space-y-[10px]">
        <div className="text-md">Welcome to Lana Bank Admin Panel</div>
        <div className="text-md font-light">Click below to sign in with Keycloak</div>
      </div>
      <div className="space-y-[20px] w-full">
        <Button onClick={handleLogin} className="w-full">
          Sign In with Keycloak
        </Button>
      </div>
    </>
  )
}

export default Login
