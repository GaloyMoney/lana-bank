"use client"

import { useState } from "react"
import { useTranslations } from "next-intl"
import { XCircle } from "lucide-react"

import { Button } from "@lana/web/ui/button"
import { formatDate } from "@lana/web/utils"

import CloseProspectDialog from "./close-prospect"

import { ProspectStatusBadge } from "@/app/prospects/prospect-status-badge"
import { DetailsCard, DetailItemProps } from "@/components/details"
import {
  CustomerType,
  GetProspectBasicDetailsQuery,
  ProspectStatus,
} from "@/lib/graphql/generated"

type ProspectDetailsCardProps = {
  prospect: NonNullable<GetProspectBasicDetailsQuery["prospectByPublicId"]>
}

export const ProspectDetailsCard: React.FC<ProspectDetailsCardProps> = ({
  prospect,
}) => {
  const t = useTranslations("Prospects.ProspectDetails.details")
  const [openCloseDialog, setOpenCloseDialog] = useState(false)

  const getCustomerTypeDisplay = (customerType: CustomerType) => {
    switch (customerType) {
      case CustomerType.Individual:
        return t("customerType.individual")
      case CustomerType.GovernmentEntity:
        return t("customerType.governmentEntity")
      case CustomerType.PrivateCompany:
        return t("customerType.privateCompany")
      case CustomerType.Bank:
        return t("customerType.bank")
      case CustomerType.FinancialInstitution:
        return t("customerType.financialInstitution")
      case CustomerType.ForeignAgencyOrSubsidiary:
        return t("customerType.foreignAgency")
      case CustomerType.NonDomiciledCompany:
        return t("customerType.nonDomiciledCompany")
      default:
        return customerType
    }
  }

  const details: DetailItemProps[] = [
    {
      label: t("labels.status"),
      value: <ProspectStatusBadge status={prospect.status} />,
    },
    {
      label: t("labels.email"),
      value: prospect.email,
    },
    {
      label: t("labels.telegram"),
      value: prospect.telegramHandle,
    },
    { label: t("labels.createdOn"), value: formatDate(prospect.createdAt) },
    {
      label: t("labels.customerType"),
      value: getCustomerTypeDisplay(prospect.customerType),
    },
  ]

  const footerContent =
    prospect.status === ProspectStatus.Open ? (
      <Button
        variant="destructive"
        onClick={() => setOpenCloseDialog(true)}
        data-testid="close-prospect-btn"
      >
        <XCircle />
        {t("buttons.closeProspect")}
      </Button>
    ) : null

  return (
    <>
      <DetailsCard
        title={t("title")}
        details={details}
        className="w-full"
        columns={4}
        footerContent={footerContent}
      />
      <CloseProspectDialog
        prospectId={prospect.prospectId}
        openCloseDialog={openCloseDialog}
        setOpenCloseDialog={setOpenCloseDialog}
      />
    </>
  )
}
