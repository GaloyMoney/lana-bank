"use client"

import React, { useState } from "react"
import Link from "next/link"
import { useTranslations } from "next-intl"
import { ArrowRight } from "lucide-react"

import { Button } from "@lana/web/ui/button"

import { DisbursalStatusBadge } from "../status-badge"

import { DetailsCard, DetailItemProps } from "@/components/details"
import Balance from "@/components/balance/balance"
import { ApprovalProcessStatus, GetDisbursalDetailsQuery } from "@/lib/graphql/generated"
import ApprovalDialog from "@/app/actions/approve"
import DenialDialog from "@/app/actions/deny"

type DisbursalDetailsProps = {
  disbursal: NonNullable<GetDisbursalDetailsQuery["disbursalByPublicId"]>
}

export const DisbursalDetailsCard: React.FC<DisbursalDetailsProps> = ({ disbursal }) => {
  const t = useTranslations("Disbursals.DisbursalDetails.DetailsCard")

  const [openApprovalDialog, setOpenApprovalDialog] = useState(false)
  const [openDenialDialog, setOpenDenialDialog] = useState(false)

  const details: DetailItemProps[] = [
    {
      label: t("details.customerEmail"),
      value: disbursal.creditFacility.customer.email,
      href: `/customers/${disbursal.creditFacility.customer.publicId}`,
    },
    {
      label: t("details.disbursalAmount"),
      value: <Balance amount={disbursal.amount} currency="usd" />,
    },
    {
      label: t("details.facilityAmount"),
      value: <Balance amount={disbursal.creditFacility.facilityAmount} currency="usd" />,
    },
    {
      label: t("details.status"),
      value: (
        <DisbursalStatusBadge
          status={disbursal.status}
          data-testid="disbursal-status-badge"
        />
      ),
    },
  ]

  const footerContent = (
    <>
      {disbursal.creditFacility.customer.depositAccount && (
        <Link
          href={`/deposit-accounts/${disbursal.creditFacility.customer.depositAccount.publicId}`}
        >
          <Button variant="outline">
            {t("buttons.viewDepositAccount")}
            <ArrowRight />
          </Button>
        </Link>
      )}
      {disbursal.approvalProcess?.status === ApprovalProcessStatus.InProgress &&
        disbursal.approvalProcess.userCanSubmitDecision && (
          <>
            <Button
              data-testid="disbursal-approve-button"
              onClick={() => setOpenApprovalDialog(true)}
              variant="outline"
            >
              {t("buttons.approve")}
            </Button>
            <Button
              data-testid="disbursal-deny-button"
              onClick={() => setOpenDenialDialog(true)}
              variant="outline"
            >
              {t("buttons.deny")}
            </Button>
          </>
        )}
      <Link href={`/credit-facilities/${disbursal.creditFacility.publicId}`}>
        <Button variant="outline">
          {t("buttons.viewCreditFacility")}
          <ArrowRight />
        </Button>
      </Link>
    </>
  )

  return (
    <>
      <DetailsCard
        title={t("title")}
        details={details}
        footerContent={footerContent}
        errorMessage={disbursal.approvalProcess.deniedReason}
      />
      <ApprovalDialog
        approvalProcess={disbursal.approvalProcess}
        openApprovalDialog={openApprovalDialog}
        setOpenApprovalDialog={() => setOpenApprovalDialog(false)}
      />
      <DenialDialog
        approvalProcess={disbursal.approvalProcess}
        openDenialDialog={openDenialDialog}
        setOpenDenialDialog={() => setOpenDenialDialog(false)}
      />
    </>
  )
}
