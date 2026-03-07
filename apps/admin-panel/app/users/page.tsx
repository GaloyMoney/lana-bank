"use client"
import { useState } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

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
  User,
  UsersSort,
  SortDirection,
  useUsersQuery,
} from "@/lib/graphql/generated"

import { camelToScreamingSnake } from "@/lib/utils"

gql`
  fragment UserFields on User {
    id
    userId
    email
    role {
      ...RoleFields
    }
    createdAt
  }

  query Users($first: Int!, $after: String, $sort: UsersSort) {
    users(first: $first, after: $after, sort: $sort) {
      pageInfo {
        hasPreviousPage
        hasNextPage
        startCursor
        endCursor
      }
      edges {
        cursor
        node {
          ...UserFields
        }
      }
    }
  }

  mutation UserUpdateRole($input: UserUpdateRoleInput!) {
    userUpdateRole(input: $input) {
      user {
        ...UserFields
      }
    }
  }
`

function UsersPage() {
  const t = useTranslations("Users")
  const [sortBy, setSortBy] = useState<UsersSort | null>(null)

  const { data, loading, fetchMore } = useUsersQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
      sort: sortBy,
    },
  })

  const columns: Column<User>[] = [
    {
      key: "email",
      label: t("table.headers.email"),
      sortable: true,
    },
    {
      key: "role",
      label: t("table.headers.role"),
      render: (role) => (
        <div>{role?.name ? <>{role.name}</> : t("table.noRolesAssigned")}</div>
      ),
    },
    {
      key: "createdAt",
      label: t("table.headers.createdAt"),
      sortable: true,
      render: (createdAt) => <DateWithTooltip value={createdAt} />,
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
          <PaginatedTable<User>
            columns={columns}
            data={data?.users as PaginatedData<User>}
            loading={loading}
            fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
            pageSize={DEFAULT_PAGESIZE}
            navigateTo={(user) => `/users/${user.userId}`}
            onSort={(column, direction) => {
              setSortBy({
                by: camelToScreamingSnake(column as string) as UsersSort["by"],
                direction: direction as SortDirection,
              })
            }}
          />
        </CardContent>
      </Card>
    </>
  )
}

export default UsersPage
