import LoanDetailsCard from "./details"

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
    </main>
  )
}

export default loanDetails
