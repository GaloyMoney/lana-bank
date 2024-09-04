"use client"

import { gql } from "@apollo/client"
import { toast } from "sonner"

import {
  Role,
  useGetUserDetailsQuery,
  useUserAssignRoleMutation,
  useUserRevokeRoleMutation,
} from "@/lib/graphql/generated"
import { DetailItem } from "@/components/details"
import { Card, CardContent, CardHeader } from "@/components/primitive/card"
import { Separator } from "@/components/primitive/separator"
import { Checkbox } from "@/components/primitive/check-box"
import { formatRole } from "@/lib/utils"
import { Badge } from "@/components/primitive/badge"

gql`
  query GetUserDetails($id: UUID!) {
    user(id: $id) {
      userId
      email
      roles
    }
  }
`

type UserDetailsProps = { userId: string }

const UserDetailsCard: React.FC<UserDetailsProps> = ({ userId }) => {
  const { data, loading, error, refetch } = useGetUserDetailsQuery({
    variables: { id: userId },
  })

  const [assignRole, { loading: assigning, error: assignRoleError }] =
    useUserAssignRoleMutation()
  const [revokeRole, { loading: revoking, error: revokeError }] =
    useUserRevokeRoleMutation()

  const handleRoleChange = async (role: Role) => {
    if (data?.user?.roles.includes(role)) {
      try {
        await revokeRole({ variables: { input: { id: userId, role } } })
        refetch()
        toast.success("Role revoked")
      } catch (err) {
        toast.error(`Failed to revoke role ,${revokeError?.message}`)
      }
    } else {
      try {
        await assignRole({ variables: { input: { id: userId, role } } })
        refetch()
        toast.success("Role assigned")
      } catch (err) {
        toast.error(`Failed to assign role, ${assignRoleError?.message}`)
      }
    }
  }

  return (
    <>
      <Card>
        {loading ? (
          <CardContent className="pt-6">Loading...</CardContent>
        ) : error ? (
          <CardContent className="pt-6 text-destructive">{error.message}</CardContent>
        ) : data ? (
          <>
            <CardHeader className="flex flex-row justify-between items-center">
              <div>
                <h2 className="font-semibold leading-none tracking-tight">User</h2>
                <p className="text-textColor-secondary text-sm mt-2">
                  {data.user?.email}
                </p>
              </div>
              {data.user?.roles.includes(Role.Superuser) && (
                <Badge variant="success" className="uppercase text-sm">
                  {formatRole(Role.Superuser)}
                </Badge>
              )}
            </CardHeader>
            <Separator className="mb-6" />
            <CardContent>
              <div className="grid grid-rows-min">
                <DetailItem label="Email" value={data.user?.email} />
                <DetailItem label="User ID" value={data.user?.userId} />
              </div>
              <div className="mt-4 grid grid-rows-min">
                <h3 className="ml-2 font-semibold leading-none tracking-tight">Roles</h3>
                <div className="ml-2 mt-4 flex space-y-1 flex-col">
                  {Object.values(Role)
                    .filter((role) => role !== Role.Superuser)
                    .map((role) => (
                      <div className="flex flex-row items-center" key={role}>
                        <Checkbox
                          id={role}
                          checked={data.user?.roles.includes(role as Role)}
                          onCheckedChange={() => handleRoleChange(role as Role)}
                          disabled={assigning || revoking}
                        />
                        <label htmlFor={role} className="ml-2">
                          {formatRole(role as Role)}
                        </label>
                      </div>
                    ))}
                </div>
              </div>
            </CardContent>
          </>
        ) : (
          userId &&
          !data && <CardContent className="pt-6">No user found with this ID</CardContent>
        )}
      </Card>
    </>
  )
}

export default UserDetailsCard
