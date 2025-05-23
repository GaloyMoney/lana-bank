"use client"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"
import { useState } from "react"

import { Badge } from "@lana/web/ui/badge"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import DataTable, { Column } from "../../components/data-table"

import { CreateRoleDialog } from "./create"
import { UpdateRoleDialog } from "./update"

import { useRolesQuery } from "@/lib/graphql/generated"

gql`
  fragment PermissionSetFields on PermissionSet {
    id
    permissionSetId
    name
  }

  fragment RoleEntityFields on RoleEntity {
    id
    roleId
    name
    permissionSets {
      ...PermissionSetFields
    }
  }

  query Roles($first: Int!, $after: String) {
    roles(first: $first, after: $after) {
      edges {
        node {
          ...RoleEntityFields
        }
      }
    }
  }
`

type Role = NonNullable<
  NonNullable<
    NonNullable<ReturnType<typeof useRolesQuery>["data"]>
  >["roles"]["edges"][number]["node"]
>

function CompactPermissionSets({
  permissionSets,
  maxShow = 7,
}: {
  permissionSets: Role["permissionSets"]
  maxShow?: number
}) {
  const t = useTranslations("RolesAndPermissions.table")

  if (!permissionSets || permissionSets.length === 0) {
    return <span className="text-muted-foreground">{t("noPermissionSetsAssigned")}</span>
  }

  const visiblePermissions = permissionSets.slice(0, maxShow)
  const remainingCount = permissionSets.length - maxShow

  return (
    <div className="flex flex-wrap gap-2 items-center">
      {visiblePermissions.map((permissionSet) => (
        <Badge variant="secondary" key={permissionSet.permissionSetId}>
          {permissionSet.name}
        </Badge>
      ))}
      {remainingCount > 0 && (
        <Badge variant="outline" className="text-muted-foreground">
          +{remainingCount} {t("morePermissions")}
        </Badge>
      )}
    </div>
  )
}

function RolesAndPermissionsPage() {
  const t = useTranslations("RolesAndPermissions")
  const [isCreateRoleDialogOpen, setIsCreateRoleDialogOpen] = useState(false)
  const [isUpdateRoleDialogOpen, setIsUpdateRoleDialogOpen] = useState(false)
  const [selectedRole, setSelectedRole] = useState<Role | null>(null)

  const { data: rolesData, loading } = useRolesQuery({
    variables: { first: 100 },
  })

  const roles = rolesData?.roles.edges.map((edge) => edge.node) || []

  const handleRoleClick = (role: Role) => {
    setSelectedRole(role)
    setIsUpdateRoleDialogOpen(true)
  }

  const columns: Column<Role>[] = [
    {
      key: "name",
      header: t("table.headers.name"),
      render: (name, role) => (
        <div className="cursor-pointer hover:text-primary transition-colors">
          <div className="font-medium">{name}</div>
          <div className="text-muted-foreground">
            {role.permissionSets.length} {t("table.permissionsCount")}
          </div>
        </div>
      ),
    },
    {
      key: "permissionSets",
      header: t("table.headers.permissionSets"),
      render: (permissionSets) => (
        <div className="cursor-pointer">
          <CompactPermissionSets permissionSets={permissionSets} maxShow={4} />
        </div>
      ),
    },
  ]

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>{t("title")}</CardTitle>
          <CardDescription>{t("description")}</CardDescription>
        </CardHeader>
        <CardContent>
          <DataTable
            data={roles}
            columns={columns}
            loading={loading}
            emptyMessage={t("table.emptyMessage")}
            onRowClick={handleRoleClick}
          />
        </CardContent>
      </Card>
      <CreateRoleDialog
        open={isCreateRoleDialogOpen}
        onOpenChange={setIsCreateRoleDialogOpen}
      />
      <UpdateRoleDialog
        open={isUpdateRoleDialogOpen}
        onOpenChange={setIsUpdateRoleDialogOpen}
        role={selectedRole}
      />
    </>
  )
}

export default RolesAndPermissionsPage
