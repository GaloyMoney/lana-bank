import React from "react"
import { FaCheckCircle } from "react-icons/fa"

import { Card } from "@/components/primitive/card"

import { GetLoanDetailsQuery } from "@/lib/graphql/generated"
import { formatDate, formatRole } from "@/lib/utils"

type LoanApproverProps = {
  approval: NonNullable<NonNullable<GetLoanDetailsQuery["loan"]>["approvals"]>[0]
}
const LoanApprover: React.FC<LoanApproverProps> = ({ approval }) => (
  <Card className="flex items-center space-x-3 p-4 mt-4">
    <FaCheckCircle className="h-6 w-6 text-green-500" />
    <div>
      <p className="text-sm font-medium">{approval.user.email}</p>
      <p className="text-sm text-textColor-secondary">
        {approval.user.roles.map(formatRole).join(", ")}
      </p>
      <p className="mt-1 text-xs text-textColor-secondary">
        Approved on {formatDate(approval.approvedAt)}
      </p>
    </div>
  </Card>
)

type LoanApproversProps = {
  loan: NonNullable<GetLoanDetailsQuery["loan"]>
}

export const LoanApprovers: React.FC<LoanApproversProps> = ({ loan }) => {
  return (
    <>
      {loan.approvals.map((approval) => (
        <LoanApprover key={approval.approvedAt} approval={approval} />
      ))}
    </>
  )
}
