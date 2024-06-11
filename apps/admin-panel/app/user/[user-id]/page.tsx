import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import { Separator } from "@/components/primitive/separator"
import {
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import { PageHeading } from "@/components/page-heading"
import { getUserByUserId } from "@/lib/graphql/query/get-user-by-userid"
import { UserActions } from "@/components/user-action"
import { getLoansForUser } from "@/lib/graphql/query/get-loans-for-user"
import { DetailItem, DetailsGroup } from "@/components/details"
import { currencyConverter, formatCurrency } from "@/lib/utils"

async function UserDetails({
  params,
}: {
  params: {
    "user-id": string
  }
}) {
  const { "user-id": userId } = params
  return (
    <main>
      <PageHeading>User</PageHeading>
      <UserDetailsCard userId={userId} />
      <UserLoansTable userId={userId} />
    </main>
  )
}

const UserDetailsCard = async ({ userId }: { userId: string }) => {
  const userDetails = await getUserByUserId({ id: userId })

  return (
    <Card>
      {userDetails instanceof Error ? (
        <CardContent className="p-6">{userDetails.message}</CardContent>
      ) : !userDetails.user ? (
        <CardContent className="p-6">No user found with this ID</CardContent>
      ) : (
        <>
          <CardHeader>
            <div className="flex justify-between items-center">
              <CardTitle>
                <div className="flex flex-col gap-1">
                  <p className="text-lg">{userDetails.user.bitfinexUsername}</p>
                  <p className="text-sm text-textColor-secondary">{userId}</p>
                </div>
              </CardTitle>
              <UserActions userId={userId} />
            </div>
            <Separator />
          </CardHeader>
          <Card>
            <CardHeader>
              <CardTitle>User details</CardTitle>
            </CardHeader>
            <CardContent>
              <DetailsGroup>
                <DetailItem label="User ID" value={userDetails.user.userId} />
                <DetailItem
                  label="Bitfinex username"
                  value={userDetails.user.bitfinexUsername}
                />
                <DetailItem
                  label="Checking Pending Balance (USD)"
                  value={formatCurrency({
                    amount: currencyConverter.centsToUsd(
                      userDetails.user.balance.checking.pending.usdBalance,
                    ),
                    currency: "USD",
                  })}
                />
                <DetailItem
                  label="Checking Settled Balance (USD)"
                  value={formatCurrency({
                    amount: currencyConverter.centsToUsd(
                      userDetails.user.balance.checking.settled.usdBalance,
                    ),
                    currency: "USD",
                  })}
                />
                <DetailItem
                  label="Uncollected Collateral Settled (BTC)"
                  value={`${userDetails.user.balance.unallocatedCollateral.settled.btcBalance} sats`}
                />
              </DetailsGroup>
            </CardContent>
          </Card>
        </>
      )}
    </Card>
  )
}

const UserLoansTable = async ({ userId }: { userId: string }) => {
  const userLoans = await getLoansForUser({ userId })

  return (
    <Card className="mt-4">
      {userLoans instanceof Error ? (
        <CardContent className="p-6">{userLoans.message}</CardContent>
      ) : !userLoans.loansForUser || userLoans.loansForUser.length === 0 ? (
        <CardContent className="p-6">No loans found for this user</CardContent>
      ) : (
        <>
          <CardHeader>
            <CardTitle>User loans</CardTitle>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableCell>Loan ID</TableCell>
                  <TableCell>Collateral (BTC)</TableCell>
                  <TableCell>Interest Incurred (USD)</TableCell>
                  <TableCell>Outstanding (USD)</TableCell>
                </TableRow>
              </TableHeader>
              <TableBody>
                {userLoans.loansForUser.map((loan) => (
                  <TableRow key={loan.loanId}>
                    <TableCell>{loan.loanId}</TableCell>
                    <TableCell>{loan.balance.collateral.btcBalance} sats</TableCell>
                    <TableCell>
                      {formatCurrency({
                        amount: currencyConverter.centsToUsd(
                          loan.balance.interestIncurred.usdBalance,
                        ),
                        currency: "USD",
                      })}
                    </TableCell>
                    <TableCell>
                      {formatCurrency({
                        amount: currencyConverter.centsToUsd(
                          loan.balance.outstanding.usdBalance,
                        ),
                        currency: "USD",
                      })}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </>
      )}
    </Card>
  )
}

export default UserDetails
