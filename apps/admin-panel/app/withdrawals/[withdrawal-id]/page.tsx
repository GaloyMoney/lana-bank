import WithdrawalDetailsCard from "./details"

import { PageHeading } from "@/components/page-heading"

function withdrawalDetails({
  params,
}: {
  params: {
    "withdrawal-id": string
  }
}) {
  const { "withdrawal-id": withdrawalId } = params

  return (
    <main>
      <PageHeading>Withdrawal Details</PageHeading>
      <WithdrawalDetailsCard withdrawalId={withdrawalId} />
    </main>
  )
}

export default withdrawalDetails
