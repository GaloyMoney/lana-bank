"use client"

import { signIn } from "next-auth/react"
import { Button } from "@lana/web/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

const KeycloakAuthForm = () => {
  const handleSignIn = async () => {
    await signIn("keycloak", {
      callbackUrl: "/",
      redirect: true,
    })
  }

  return (
    <Card className="md:w-2/5 my-2" variant="transparent">
      <CardHeader>
        <CardTitle>Welcome to Lana Bank</CardTitle>
        <CardDescription>
          Sign in with your Lana Bank account to access your customer portal.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <p className="text-sm text-muted-foreground mb-4">
          You will be redirected to our secure login page to authenticate with your
          account.
        </p>
      </CardContent>
      <CardFooter>
        <Button
          data-test-id="keycloak-signin-btn"
          onClick={handleSignIn}
          className="w-full"
        >
          Sign In
        </Button>
      </CardFooter>
    </Card>
  )
}

export { KeycloakAuthForm }
