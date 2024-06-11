import { redirect } from "next/navigation"

import Link from "next/link"

import { Input } from "@/components/primitive/input"
import { GetUserByUserIdQuery, UsersQuery } from "@/lib/graphql/generated"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import { Card, CardContent } from "@/components/primitive/card"
import { Label } from "@/components/primitive/label"
import { Button } from "@/components/primitive/button"
import { EllipsisHorizontal } from "@/components/icons"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/primitive/dropdown-menu"
import { PageHeading } from "@/components/page-heading"
import { getUserByUserId } from "@/lib/graphql/query/get-user-by-userid"
import { getUsers } from "@/lib/graphql/query/get-users"
import { currencyConverter, formatCurrency } from "@/lib/utils"

const searchUser = async (formData: FormData) => {
  "use server"
  if (formData.get("submit") === "clear") {
    redirect(`/user`)
  }

  const userId = formData.get("userId")
  if (!userId || typeof userId !== "string") {
    redirect(`/user`)
  }
  redirect(`/user?userId=${userId}`)
}

async function UserPage({ searchParams }: { searchParams: { userId?: string } }) {
  const { userId } = searchParams
  let userDetails: GetUserByUserIdQuery["user"][] | UsersQuery["users"]["nodes"] | null =
    null
  let error: Error | null = null

  if (userId) {
    const result = await getUserByUserId({ id: userId })
    if (result instanceof Error) {
      error = result
    } else {
      userDetails = result.user ? [result.user] : null
    }
  } else {
    const result = await getUsers({ first: 100 })
    if (result instanceof Error) {
      error = result
    } else {
      userDetails = result.users.nodes ? result.users.nodes : null
    }
  }

  return (
    <main>
      <PageHeading>User</PageHeading>
      <div className="mt-4 mb-4 max-w-[30rem]">
        <Label htmlFor="userId">User ID</Label>
        <form className="flex gap-2" action={searchUser}>
          <Input placeholder="Find a user by user ID" id="userId" name="userId" />
          <Button variant="secondary">Search</Button>
          {userId && (
            <Button type="submit" name="submit" value="clear">
              X Clear
            </Button>
          )}
        </form>
      </div>
      {error ? (
        <Card>
          <CardContent className="p-4">
            <div>{error.message}</div>
          </CardContent>
        </Card>
      ) : (
        <Card>
          <CardContent className="pt-6">
            {userDetails && userDetails.length > 0 ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>User</TableHead>
                    <TableHead>BTC Balance (Settled)</TableHead>
                    <TableHead>USD Balance (Settled)</TableHead>
                    <TableHead>USD Balance (Pending)</TableHead>
                    <TableHead></TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {userDetails.map((user) =>
                    user ? (
                      <TableRow key={user.userId}>
                        <TableCell>
                          <div className="flex flex-col gap-1">
                            <div>{user.bitfinexUsername}</div>
                            <div className="text-xs text-textColor-secondary">
                              {user.userId}
                            </div>
                          </div>
                        </TableCell>
                        <TableCell>
                          {user.balance.unallocatedCollateral.settled?.btcBalance} sats
                        </TableCell>
                        <TableCell>
                          {formatCurrency({
                            amount: currencyConverter.centsToUsd(
                              user.balance.checking.settled?.usdBalance,
                            ),
                            currency: "USD",
                          })}
                        </TableCell>
                        <TableCell>
                          {formatCurrency({
                            amount: currencyConverter.centsToUsd(
                              user.balance.checking.pending?.usdBalance,
                            ),
                            currency: "USD",
                          })}
                        </TableCell>
                        <TableCell>
                          <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                              <Button variant="ghost">
                                <EllipsisHorizontal className="w-4 h-4" />
                              </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent className="text-sm">
                              <Link href={`/user/${user.userId}`}>
                                <DropdownMenuItem>View details</DropdownMenuItem>
                              </Link>
                            </DropdownMenuContent>
                          </DropdownMenu>
                        </TableCell>
                      </TableRow>
                    ) : null,
                  )}
                </TableBody>
              </Table>
            ) : (
              <div>No data available</div>
            )}
          </CardContent>
        </Card>
      )}
    </main>
  )
}

export default UserPage
