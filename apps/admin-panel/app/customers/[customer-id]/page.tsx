import { CustomerDetailsCard } from "./details"
import { CustomerLoansTable } from "./loans"
import { CustomerDepositsTable } from "./deposits"
import { CustomerWithdrawalsTable } from "./withdrawals"

import { PageHeading } from "@/components/page-heading"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/primitive/tab"

function customerDetails({
  params,
}: {
  params: {
    "customer-id": string
  }
}) {
  const { "customer-id": customerId } = params
  return (
    <main>
      <PageHeading>Customer Details</PageHeading>
      <CustomerDetailsCard customerId={customerId} />
      <Tabs defaultValue="loans" className="mt-4">
        <TabsList>
          <TabsTrigger value="loans">Loans</TabsTrigger>
          <TabsTrigger value="deposit">Deposits</TabsTrigger>
          <TabsTrigger value="withdrawals">Withdrawals</TabsTrigger>
        </TabsList>
        <TabsContent value="loans">
          <CustomerLoansTable customerId={customerId} />
        </TabsContent>
        <TabsContent value="deposit">
          <CustomerDepositsTable customerId={customerId} />
        </TabsContent>
        <TabsContent value="withdrawals">
          <CustomerWithdrawalsTable customerId={customerId} />
        </TabsContent>
      </Tabs>
    </main>
  )
}

export default customerDetails
