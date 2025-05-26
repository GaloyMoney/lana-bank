"use client"

import React, { useEffect, use } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { RoleDetailsCard } from "./details"

import { useBreadcrumb } from "@/app/breadcrumb-provider"
import { useRoleQuery } from "@/lib/graphql/generated"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"

gql`
  query Role($id: UUID!) {
    role(id: $id) {
      ...RoleFields
    }
  }
`

function RolePage({
  params,
}: {
  params: Promise<{
    "role-id": string
  }>
}) {
  const { "role-id": roleId } = use(params)
  const { setCustomLinks, resetToDefault } = useBreadcrumb()
  const navTranslations = useTranslations("Sidebar.navItems")

  const { data, loading, error } = useRoleQuery({
    variables: { id: roleId },
  })

  useEffect(() => {
    if (data?.role) {
      setCustomLinks([
        { title: navTranslations("rolesAndPermissions"), href: "/roles-and-permissions" },
        { title: data.role.name, isCurrentPage: true },
      ])
    }

    return () => {
      resetToDefault()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data?.role])

  if (loading && !data) {
    return <DetailsPageSkeleton tabs={0} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.role) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto">
      <RoleDetailsCard role={data.role} />
    </main>
  )
}

export default RolePage
