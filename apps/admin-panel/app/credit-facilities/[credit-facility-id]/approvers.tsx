import React from "react"
import { FaCheckCircle } from "react-icons/fa"

import { Card } from "@/components/primitive/card"

import { GetCreditFacilityDetailsQuery } from "@/lib/graphql/generated"
import { formatDate } from "@/lib/utils"

type CreditFacilityApproverProps = {
  approval: NonNullable<
    NonNullable<GetCreditFacilityDetailsQuery["creditFacility"]>["approvals"]
  >[0]
}

const CreditFacilityApprover: React.FC<CreditFacilityApproverProps> = ({ approval }) => (
  <Card className="flex items-center space-x-3 p-4 mt-4">
    <FaCheckCircle className="h-6 w-6 text-green-500" />
    <div>
      <p className="text-sm font-medium">User ID: {approval.userId}</p>
      <p className="mt-1 text-xs text-textColor-secondary">
        Approved on {formatDate(approval.approvedAt)}
      </p>
    </div>
  </Card>
)

type CreditFacilityApproversProps = {
  creditFacility: NonNullable<GetCreditFacilityDetailsQuery["creditFacility"]>
}

export const CreditFacilityApprovers: React.FC<CreditFacilityApproversProps> = ({
  creditFacility,
}) => {
  return (
    <>
      {creditFacility.approvals.map((approval) => (
        <CreditFacilityApprover key={approval.approvedAt} approval={approval} />
      ))}
    </>
  )
}
