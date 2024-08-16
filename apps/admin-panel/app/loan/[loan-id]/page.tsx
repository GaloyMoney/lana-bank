import LoanDetailsCard from "./loan-details"

import { PageHeading } from "@/components/page-heading"

function loanDetails({
  params,
}: {
  params: {
    "loan-id": string
  }
}) {
  const { "loan-id": loanId } = params

  return (
    <main>
      <PageHeading>Loan Details</PageHeading>
      <LoanDetailsCard loanId={loanId} />
      {/* <Tabs defaultValue="loans" className="mt-4">
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
      </Tabs> */}
    </main>
  )
}

export default loanDetails
