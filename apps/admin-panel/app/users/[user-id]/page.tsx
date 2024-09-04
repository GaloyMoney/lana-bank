import UserDetailsCard from "./details"

import { PageHeading } from "@/components/page-heading"

function userDetails({
  params,
}: {
  params: {
    "user-id": string
  }
}) {
  const { "user-id": userId } = params

  return (
    <main>
      <PageHeading>User Details</PageHeading>
      <UserDetailsCard userId={userId} />
    </main>
  )
}

export default userDetails
