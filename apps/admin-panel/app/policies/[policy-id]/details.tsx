"use client"
import { useTranslations } from "next-intl"
import React from "react"

import { Button } from "@lana/web/ui/button"

import { CommitteeAssignmentDialog } from "./assign-to-committee"

import { DetailsCard, DetailItemProps } from "@/components/details"
import { ApprovalRules, GetPolicyDetailsQuery } from "@/lib/graphql/generated"
import { useProcessTypeLabel, useRuleLabel } from "@/app/actions/hooks"

type PolicyDetailsProps = {
  policy: NonNullable<GetPolicyDetailsQuery["policy"]>
}

export const PolicyDetailsCard: React.FC<PolicyDetailsProps> = ({ policy }) => {
  const t = useTranslations("Policies.PolicyDetails.DetailsCard")
  const processTypeLabel = useProcessTypeLabel()
  const ruleLabel = useRuleLabel()

  const [openAssignDialog, setOpenAssignDialog] = React.useState(false)
  const policyRuleType = policy.rules.__typename

  const details: DetailItemProps[] = [
    {
      label: t("fields.processType"),
      value: processTypeLabel(policy.approvalProcessType),
    },
    {
      label: t("fields.rule"),
      value: ruleLabel(policy.rules as ApprovalRules),
    },
    ...(policyRuleType === "CommitteeApproval"
      ? [
          {
            label: t("fields.assignedCommittee"),
            value: policy.rules.committee.name,
          },
        ]
      : []),
  ]

  const footerContent = (
    <Button
      variant="outline"
      onClick={() => setOpenAssignDialog(true)}
      data-testid="policy-assign-committee"
    >
      {policyRuleType === "CommitteeApproval"
        ? t("buttons.updatePolicy")
        : t("buttons.assignCommittee")}
    </Button>
  )

  return (
    <>
      <DetailsCard
        title={t("title")}
        details={details}
        footerContent={footerContent}
        className="w-full"
      />

      <CommitteeAssignmentDialog
        policyId={policy.policyId}
        openAssignDialog={openAssignDialog}
        setOpenAssignDialog={setOpenAssignDialog}
      />
    </>
  )
}
