"use client"
import React from "react"

import { CommitteeAssignmentDialog } from "./assign-to-committee"

import { ApprovalRules, GetPolicyDetailsQuery } from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/primitive/card"
import { Button } from "@/components/primitive/button"
import { formatRule, formatProcessType } from "@/lib/utils"

type PolicyDetailsProps = {
  policy: NonNullable<GetPolicyDetailsQuery["policy"]>
}

export const PolicyDetailsCard: React.FC<PolicyDetailsProps> = ({ policy }) => {
  const [openAssignDialog, setOpenAssignDialog] = React.useState(false)
  const policyRuleType = policy.rules.__typename

  return (
    <div className="flex">
      <Card className="w-full">
        <CardHeader className="flex-row justify-between items-center">
          <CardTitle>Policy</CardTitle>
        </CardHeader>
        <CardContent>
          <DetailsGroup>
            <DetailItem
              label="Process Type"
              value={formatProcessType(policy.approvalProcessType)}
            />
            <DetailItem label="Rule" value={formatRule(policy.rules as ApprovalRules)} />
            {policyRuleType === "CommitteeThreshold" && (
              <DetailItem
                label="Assigned Committee"
                value={policy.rules.committee.name}
              />
            )}
          </DetailsGroup>
        </CardContent>
      </Card>

      <div className="flex flex-col space-y-2 mt-1 ml-4">
        <Button
          variant="primary"
          className="w-full"
          onClick={() => setOpenAssignDialog(true)}
        >
          {policyRuleType === "CommitteeThreshold" ? "Update Policy" : "Assign Committee"}
        </Button>
      </div>

      <CommitteeAssignmentDialog
        policyId={policy.policyId}
        openAssignDialog={openAssignDialog}
        setOpenAssignDialog={setOpenAssignDialog}
      />
    </div>
  )
}
