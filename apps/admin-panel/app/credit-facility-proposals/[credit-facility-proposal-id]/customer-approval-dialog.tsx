"use client"

import React, { useState } from "react"
import { gql, useApolloClient } from "@apollo/client"
import { toast } from "sonner"
import { useTranslations } from "next-intl"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@lana/web/ui/dialog"
import { Button } from "@lana/web/ui/button"
import { formatDate } from "@lana/web/utils"

import {
  GetCreditFacilityProposalLayoutDetailsDocument,
  useCreditFacilityProposalCustomerApprovalConcludeMutation,
} from "@/lib/graphql/generated"
import { DetailItem, DetailsGroup } from "@/components/details"

gql`
  mutation CreditFacilityProposalCustomerApprovalConclude(
    $input: CreditFacilityProposalCustomerApprovalConcludeInput!
  ) {
    creditFacilityProposalCustomerApprovalConclude(input: $input) {
      creditFacilityProposal {
        ...CreditFacilityProposalLayoutFragment
      }
    }
  }
`

type CustomerApprovalDialogProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
  proposalId: string
  approved: boolean
  facilityAmount: string
  customerEmail: string
  createdAt: string
}

export const CustomerApprovalDialog: React.FC<CustomerApprovalDialogProps> = ({
  open,
  onOpenChange,
  proposalId,
  approved,
  facilityAmount,
  customerEmail,
  createdAt,
}) => {
  const t = useTranslations(
    "CreditFacilityProposals.ProposalDetails.CustomerApprovalDialog",
  )
  const tCommon = useTranslations("Common")

  const [error, setError] = useState<string | null>(null)
  const client = useApolloClient()

  const [concludeApproval, { loading }] =
    useCreditFacilityProposalCustomerApprovalConcludeMutation()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    try {
      await concludeApproval({
        variables: {
          input: {
            proposalId,
            approved,
          },
        },
        onCompleted: async () => {
          await client.refetchQueries({
            include: [GetCreditFacilityProposalLayoutDetailsDocument],
          })
          toast.success(approved ? t("success.approved") : t("success.denied"))
        },
      })
      onOpenChange(false)
    } catch (error) {
      if (error instanceof Error) {
        setError(error.message)
      } else {
        setError(t("errors.unknown"))
      }
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{approved ? t("title.approve") : t("title.deny")}</DialogTitle>
          <DialogDescription>
            {approved ? t("description.approve") : t("description.deny")}
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <DetailsGroup layout="horizontal">
            <DetailItem label={t("fields.customer")} value={customerEmail} />
            <DetailItem label={t("fields.facilityAmount")} value={facilityAmount} />
            <DetailItem label={t("fields.createdAt")} value={formatDate(createdAt)} />
          </DetailsGroup>
          {error && <p className="text-destructive">{error}</p>}
          <DialogFooter>
            <Button
              type="button"
              variant="ghost"
              onClick={() => onOpenChange(false)}
              disabled={loading}
            >
              {tCommon("cancel")}
            </Button>
            <Button
              type="submit"
              loading={loading}
              data-testid={`customer-approval-${approved ? "approve" : "deny"}-button`}
            >
              {approved ? t("buttons.approve") : t("buttons.deny")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
