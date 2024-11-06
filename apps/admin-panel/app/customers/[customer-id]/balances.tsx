"use client"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import Balance from "@/components/balance/balance"

import { GetCustomerQuery } from "@/lib/graphql/generated"
import { DetailItem } from "@/components/details"

type CustomerAccountBalancesProps = {
  balance: NonNullable<GetCustomerQuery["customer"]>["balance"]
}

export const CustomerAccountBalances: React.FC<CustomerAccountBalancesProps> = ({
  balance,
}) => (
  <Card className="mt-4">
    <CardHeader>
      <CardTitle>Account Balances</CardTitle>
    </CardHeader>
    <CardContent>
      <DetailItem
        label="Checking Settled Balance (USD)"
        value={<Balance amount={balance.checking.settled} currency="usd" />}
      />
      <DetailItem
        label="Pending Withdrawals (USD)"
        value={<Balance amount={balance.checking.pending} currency="usd" />}
      />
    </CardContent>
  </Card>
)
