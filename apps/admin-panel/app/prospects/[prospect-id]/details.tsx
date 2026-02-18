"use client"

import { useState } from "react"
import { useTranslations } from "next-intl"
import { XCircle, UserCheck, ArrowRight } from "lucide-react"
import Link from "next/link"

import { Button } from "@lana/web/ui/button"
import { formatDate } from "@lana/web/utils"

import CloseProspectDialog from "./close-prospect"
import ConvertProspectDialog from "./convert-prospect"

import { ProspectStageBadge } from "@/app/prospects/prospect-stage-badge"
import { DetailsCard, DetailItemProps } from "@/components/details"
import {
  GetProspectBasicDetailsQuery,
  ProspectStage,
  ProspectStatus,
  useDomainConfigsQuery,
} from "@/lib/graphql/generated"
import { CustomerTypeBadge } from "@/app/customers/customer-type-badge"

type ProspectDetailsCardProps = {
  prospect: NonNullable<GetProspectBasicDetailsQuery["prospectByPublicId"]>
}

export const ProspectDetailsCard: React.FC<ProspectDetailsCardProps> = ({
  prospect,
}) => {
  const t = useTranslations("Prospects.ProspectDetails.details")
  const [openCloseDialog, setOpenCloseDialog] = useState(false)
  const [openConvertDialog, setOpenConvertDialog] = useState(false)

  const { data: domainConfigsData } = useDomainConfigsQuery({
    variables: { first: 100 },
  })
  const requireVerifiedCustomer = domainConfigsData?.domainConfigs.nodes.find(
    (c) => c.key === "require-verified-customer-for-account",
  )
  const showConvertButton =
    requireVerifiedCustomer?.isSet && String(requireVerifiedCustomer.value) === "false"

  const personalInfo = prospect.personalInfo

  const details: DetailItemProps[] = [
    {
      label: t("labels.stage"),
      value: <ProspectStageBadge stage={prospect.stage} />,
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
      value: <CustomerTypeBadge customerType={prospect.customerType} />,
    },
    {
      label: t("labels.firstName"),
      value: personalInfo?.firstName ?? "-",
    },
    {
      label: t("labels.lastName"),
      value: personalInfo?.lastName ?? "-",
    },
    ...(personalInfo?.dateOfBirth
      ? [{ label: t("labels.dateOfBirth"), value: personalInfo.dateOfBirth }]
      : []),
    ...(personalInfo?.nationality
      ? [{ label: t("labels.nationality"), value: personalInfo.nationality }]
      : []),
    ...(personalInfo?.address
      ? [{ label: t("labels.address"), value: personalInfo.address }]
      : []),
    ...(prospect.customer
      ? [
          {
            label: t("labels.customer"),
            value: prospect.customer.email,
            href: `/customers/${prospect.customer.publicId}`,
          },
        ]
      : []),
  ]

  const footerContent =
    prospect.status === ProspectStatus.Converted && prospect.customer ? (
      <Button variant="outline" data-testid="view-customer-btn" asChild>
        <Link href={`/customers/${prospect.customer.publicId}`}>
          {t("buttons.viewCustomer")}
          <ArrowRight className="h-4 w-4 ml-2" />
        </Link>
      </Button>
    ) : prospect.stage !== ProspectStage.Converted && prospect.stage !== ProspectStage.Closed ? (
      <div className="flex gap-2">
        {showConvertButton && (
          <Button
            variant="outline"
            onClick={() => setOpenConvertDialog(true)}
            data-testid="convert-prospect-btn"
          >
            <UserCheck />
            {t("buttons.convertToCustomer")}
          </Button>
        )}
        <Button
          variant="destructive"
          onClick={() => setOpenCloseDialog(true)}
          data-testid="close-prospect-btn"
        >
          <XCircle />
          {t("buttons.closeProspect")}
        </Button>
      </div>
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
      <ConvertProspectDialog
        prospectId={prospect.prospectId}
        openConvertDialog={openConvertDialog}
        setOpenConvertDialog={setOpenConvertDialog}
      />
    </>
  )
}
