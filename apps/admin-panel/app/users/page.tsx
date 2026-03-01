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

import DataTable, { Column } from "../../components/data-table"

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
  NonNullable<NonNullable<ReturnType<typeof useUsersQuery>["data"]>["users"]>["edges"]
>[number]["node"]

function UsersPage() {
  const t = useTranslations("Users")

  const { data: usersList, loading } = useUsersQuery({ variables: { first: 100 } })

  const columns: Column<User>[] = [
    {
      key: "email",
      header: t("table.headers.email"),
    },
    {
      key: "role",
      header: t("table.headers.role"),
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
          <DataTable
            data={
              usersList?.users?.edges
                ?.map((edge) => edge?.node)
                .filter(Boolean) as User[] || []
            }
            columns={columns}
            loading={loading}
            emptyMessage={t("table.emptyMessage")}
            navigateTo={(user) => `/users/${user.userId}`}
          />
        </CardContent>
      </Card>
    </>
  )
}

export default UsersPage
