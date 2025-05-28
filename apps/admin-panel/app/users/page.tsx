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

  query Users {
    users {
      ...UserFields
    }
  }

  mutation UserUpdateRole($input: UserUpdateRoleInput!) {
    userUpdateRole(input: $input) {
      user {
        ...UserFields
      }
    }
  }

  mutation UserRevokeRole($input: UserRevokeRoleInput!) {
    userRevokeRole(input: $input) {
      user {
        ...UserFields
      }
    }
  }
`

type User = NonNullable<
  NonNullable<ReturnType<typeof useUsersQuery>["data"]>
>["users"][number]

function UsersPage() {
  const t = useTranslations("Users")

  const { data: usersList, loading } = useUsersQuery()

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
            data={usersList?.users || []}
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
