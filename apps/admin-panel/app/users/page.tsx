"use client"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "../../components/paginated-table"

import { useUsersQuery } from "@/lib/graphql/generated"

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

  query Users($first: Int!, $after: String) {
    users(first: $first, after: $after) {
      edges {
        node {
          ...UserFields
        }
        cursor
      }
      pageInfo {
        hasNextPage
        endCursor
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

type User = NonNullable<
  NonNullable<ReturnType<typeof useUsersQuery>["data"]>["users"]["edges"]
>[number]["node"]

function UsersPage() {
  const t = useTranslations("Users")

  const {
    data: usersList,
    loading,
    fetchMore,
    error,
  } = useUsersQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
  })

  const columns: Column<User>[] = [
    {
      key: "email",
      label: t("table.headers.email"),
    },
    {
      key: "role",
      label: t("table.headers.role"),
      render: (role) => (
        <div>{role?.name ? <>{role.name}</> : t("table.noRolesAssigned")}</div>
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
          {error && <p className="text-destructive text-sm">{error?.message}</p>}
          <PaginatedTable<User>
            data={usersList?.users as PaginatedData<User>}
            columns={columns}
            loading={loading}
            fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
            pageSize={DEFAULT_PAGESIZE}
            noDataText={t("table.emptyMessage")}
            navigateTo={(user) => `/users/${user.userId}`}
          />
        </CardContent>
      </Card>
    </>
  )
}

export default UsersPage
