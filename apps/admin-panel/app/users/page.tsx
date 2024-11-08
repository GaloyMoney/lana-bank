"use client"
import { gql } from "@apollo/client"

import { useState } from "react"
import Link from "next/link"
import { IoEllipsisHorizontal } from "react-icons/io5"
import { toast } from "sonner"
import { useRouter } from "next/navigation"

import {
  GetUserDetailsDocument,
  Role,
  useUserAssignRoleMutation,
  useUserRevokeRoleMutation,
  useUsersQuery,
} from "@/lib/graphql/generated"
import { formatRole } from "@/lib/utils"

import { Button } from "@/components/primitive/button"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/primitive/dropdown-menu"
import { Badge } from "@/components/primitive/badge"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/primitive/card"
import { TableLoadingSkeleton } from "@/components/table-loading-skeleton"

gql`
  query Users {
    users {
      userId
      email
      roles
    }
  }

  mutation UserAssignRole($input: UserAssignRoleInput!) {
    userAssignRole(input: $input) {
      user {
        userId
        email
        roles
      }
    }
  }

  mutation UserRevokeRole($input: UserRevokeRoleInput!) {
    userRevokeRole(input: $input) {
      user {
        userId
        email
        roles
      }
    }
  }
`

function UsersPage() {
  const router = useRouter()
  const { data: usersList, refetch, loading } = useUsersQuery()

  return (
    <Card>
      <CardHeader>
        <CardTitle>Users</CardTitle>
        <CardDescription>Manage system users and their role assignments</CardDescription>
      </CardHeader>
      <CardContent>
        {loading ? (
          <TableLoadingSkeleton />
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-1/3">Email</TableHead>
                <TableHead className="w-1/3">Roles</TableHead>
                <TableHead className="w-1/3"></TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {usersList?.users.map((user) => (
                <TableRow
                  key={user.userId}
                  className="cursor-pointer"
                  onClick={() => router.push(`/users/${user.userId}`)}
                >
                  <TableCell>{user.email}</TableCell>
                  <TableCell>
                    <div className="flex flex-wrap gap-2 text-muted-foreground items-center">
                      {user.roles.length > 0
                        ? user.roles.map((role) => (
                            <Badge variant="secondary" key={role}>
                              {formatRole(role)}
                            </Badge>
                          ))
                        : "No roles Assigned"}
                    </div>
                  </TableCell>
                  <TableCell className="text-right pr-8">
                    <RolesDropDown
                      refetch={refetch}
                      userId={user.userId}
                      roles={user.roles}
                    />
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </CardContent>
    </Card>
  )
}

export default UsersPage

const RolesDropDown = ({
  userId,
  roles,
  refetch,
}: {
  userId: string
  roles: Role[]
  refetch: () => void
}) => {
  const [checkedRoles, setCheckedRoles] = useState<Role[]>(roles)
  const [assignRole, { loading: assigning, error: assignRoleError }] =
    useUserAssignRoleMutation({
      refetchQueries: [GetUserDetailsDocument],
    })
  const [revokeRole, { loading: revoking, error: revokeError }] =
    useUserRevokeRoleMutation({
      refetchQueries: [GetUserDetailsDocument],
    })

  const handleRoleChange = async (role: Role) => {
    if (checkedRoles.includes(role)) {
      try {
        await revokeRole({ variables: { input: { id: userId, role } } })
        setCheckedRoles((prev) => prev.filter((r) => r !== role))
        refetch()
        toast.success("Role revoked")
      } catch (err) {
        toast.error(`Failed to revoke role ,${revokeError?.message}`)
      }
    } else {
      try {
        await assignRole({ variables: { input: { id: userId, role } } })
        setCheckedRoles((prev) => [...prev, role])
        refetch()
        toast.success("Role assigned")
      } catch (err) {
        toast.error(`Failed to assign role, ${assignRoleError?.message}`)
      }
    }
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger>
        <Button variant="ghost">
          <IoEllipsisHorizontal className="w-4 h-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        <Link href={`/users/${userId}`}>
          <DropdownMenuItem>View details</DropdownMenuItem>
        </Link>
        <DropdownMenuLabel>Roles</DropdownMenuLabel>
        <DropdownMenuSeparator />
        {Object.values(Role)
          .filter((role) => role !== Role.Superuser)
          .map((role) => (
            <DropdownMenuCheckboxItem
              key={role}
              checked={checkedRoles.includes(role)}
              onCheckedChange={() => handleRoleChange(role)}
              disabled={assigning || revoking}
            >
              {formatRole(role)}
            </DropdownMenuCheckboxItem>
          ))}
        {(assigning || revoking) && <div>Loading...</div>}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
