"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { Badge } from "@lana/web/ui/badge"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import DataTable, { Column } from "../../components/data-table"

import { useRolesQuery } from "@/lib/graphql/generated"
import { usePermissionDisplay } from "@/hooks/use-permission-display"

gql`
  fragment PermissionSetFields on PermissionSet {
    id
    permissionSetId
    name
    description
  }

  fragment RoleFields on Role {
    id
    roleId
    name
    createdAt
    permissionSets {
      ...PermissionSetFields
    }
  }

  query Roles($first: Int!, $after: String) {
    roles(first: $first, after: $after) {
      edges {
        node {
          ...RoleFields
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
  const { getTranslation } = usePermissionDisplay()

  if (!permissionSets || permissionSets.length === 0) {
    return <span className="text-muted-foreground">{t("noPermissionSetsAssigned")}</span>
  }

  const sortedPermissionSets = [...permissionSets].sort((a, b) =>
    a.name.localeCompare(b.name),
  )
  const visiblePermissions = sortedPermissionSets.slice(0, maxShow)
  const remainingCount = sortedPermissionSets.length - maxShow

  return (
    <div className="flex flex-wrap gap-2 items-center">
      {visiblePermissions.map((permissionSet) => (
        <Badge variant="outline" key={permissionSet.permissionSetId}>
          {getTranslation(permissionSet.name).label}
        </Badge>
      ))}
      {remainingCount > 0 && (
        <Badge variant="secondary" className="text-muted-foreground">
          +{remainingCount} {t("morePermissions")}
        </Badge>
      )}
    </div>
  )
}

function RolesAndPermissionsPage() {
  const t = useTranslations("RolesAndPermissions")

  const {
    data: rolesData,
    loading,
    error,
  } = useRolesQuery({
    variables: { first: 100 },
  })

  const roles = rolesData?.roles.edges.map((edge) => edge.node) || []

  const columns: Column<Role>[] = [
    {
      key: "name",
      header: t("table.headers.name"),
      render: (name, role) => (
        <div>
          <div className="font-medium">{name}</div>
          <div className="text-muted-foreground">
            {role.permissionSets.length} {t("table.permissionsCount")}
          </div>
        </div>
      ),
    },
    {
      key: "createdAt",
      header: t("table.headers.createdAt"),
      render: (createdAt) => <DateWithTooltip value={createdAt} />,
    },
    {
      key: "permissionSets",
      width: "60%",
      header: t("table.headers.permissionSets"),
      render: (permissionSets) => (
        <CompactPermissionSets permissionSets={permissionSets} maxShow={4} />
      ),
    },
  ]

  if (error) {
    return <div className="text-destructive">{error.message}</div>
  }

  return (
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
          navigateTo={(role) => `/roles-and-permissions/${role.roleId}`}
        />
      </CardContent>
    </Card>
  )
}

export default RolesAndPermissionsPage
