"use client"

import { useSession, signOut } from "next-auth/react"

import { Button } from "@/components/primitive/button"
import { PageHeading } from "@/components/page-heading"
import { useMeQuery } from "@/lib/graphql/generated"
import { formatRole } from "@/lib/utils"

export default function ProfilePage() {
  const { data: session, status } = useSession()
  const { data: user } = useMeQuery()

  if (status === "authenticated") {
    return (
      <main>
        <PageHeading>Profile</PageHeading>
        <div className="mb-4">
          <p className="text-lg">
            Signed in as <span className="font-semibold">{session?.user?.email}</span>
          </p>
          {user?.me && (
            <p className="text-lg">
              Role:{" "}
              <span className="font-semibold">
                {user.me.user.roles.map((role) => formatRole(role)).join(", ")}
              </span>
            </p>
          )}
        </div>
        <Button variant="primary" onClick={() => signOut()}>
          Log out
        </Button>
      </main>
    )
  }

  return (
    <main className="p-6 min-h-screen bg-gray-100">
      <div className="max-w-lg mx-auto bg-white shadow-md rounded-lg p-6 text-center">
        <h1 className="text-2xl font-bold mb-4">You are not signed in</h1>
        <a href="/api/auth/signin" className="text-blue-500 hover:underline">
          Sign in
        </a>
      </div>
    </main>
  )
}
