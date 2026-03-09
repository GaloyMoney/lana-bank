"use client"

import React, { useState, useCallback } from "react"
import Link from "next/link"
import { useTranslations } from "next-intl"

import { useGetUserDetailsLazyQuery } from "@/lib/graphql/generated"

type UserInfo = {
  userId: string
  email: string
}

export const AuditUser: React.FC<{ subjectId: string }> = ({ subjectId }) => {
  const t = useTranslations("Common")
  const [user, setUser] = useState<UserInfo | null>(null)

  const [fetchUser, { loading }] = useGetUserDetailsLazyQuery({
    fetchPolicy: "cache-first",
  })

  const handleShowUser = useCallback(
    async (e: React.MouseEvent) => {
      e.stopPropagation()
      const id = subjectId.startsWith("user:") ? subjectId.slice(5) : subjectId
      const { data } = await fetchUser({ variables: { id } })
      if (data?.user) {
        setUser({ userId: data.user.userId, email: data.user.email })
      }
    },
    [fetchUser, subjectId],
  )

  if (user) {
    return (
      <Link
        href={`/users/${user.userId}`}
        className="text-primary underline underline-offset-4 hover:text-primary/80 text-xs"
        onClick={(e) => e.stopPropagation()}
      >
        {user.email}
      </Link>
    )
  }

  return (
    <button
      type="button"
      className="text-xs text-primary hover:underline"
      onClick={handleShowUser}
      disabled={loading}
    >
      {loading ? t("loading") : t("showUser")}
    </button>
  )
}
