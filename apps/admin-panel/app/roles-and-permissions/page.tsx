"use client"

import { useState } from "react"
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

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "../../components/paginated-table"

import {
  RolesSort,
  SortDirection,
  useRolesQuery,
} from "@/lib/graphql/generated"

import { usePermissionDisplay } from "@/hooks/use-permission-display"
import { camelToScreamingSnake } from "@/lib/utils"

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

  query Roles($first: Int!, $after: String, $sort: RolesSort) {
    roles(first: $first, after: $after, sort: $sort) {
      pageInfo {
        hasPreviousPage
        hasNextPage
        startCursor
        endCursor
      }
      edges {
        cursor
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
  const [sortBy, setSortBy] = useState<RolesSort | null>(null)

  const { data, loading, fetchMore } = useRolesQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
    },
  })

  const columns: Column<Role>[] = [
    {
      key: "name",
      label: t("table.headers.name"),
      sortable: true,
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
      label: t("table.headers.createdAt"),
      sortable: true,
      render: (createdAt) => <DateWithTooltip value={createdAt} />,
    },
    {
      key: "permissionSets",
      label: t("table.headers.permissionSets"),
      render: (permissionSets) => (
        <CompactPermissionSets permissionSets={permissionSets} maxShow={4} />
      ),
    },
  ]

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <PaginatedTable<Role>
          columns={columns}
          data={data?.roles as PaginatedData<Role>}
          loading={loading}
          fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
          pageSize={DEFAULT_PAGESIZE}
          navigateTo={(role) => `/roles-and-permissions/${role.roleId}`}
          onSort={(column, direction) => {
            setSortBy({
              by: camelToScreamingSnake(column as string) as RolesSort["by"],
              direction: direction as SortDirection,
            })
          }}
        />
      </CardContent>
    </Card>
  )
}

export default RolesAndPermissionsPage
