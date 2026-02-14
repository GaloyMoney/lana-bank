"use client"

import React from "react"
import { gql } from "@apollo/client"
import { HiLink } from "react-icons/hi"

import { Copy } from "lucide-react"
import { useTranslations } from "next-intl"
import { toast } from "sonner"

import { KycStatusBadge } from "@/app/prospects/kyc-status-badge"

import {
  KycLevel,
  KycStatus,
  useSumsubPermalinkCreateMutation,
} from "@/lib/graphql/generated"
import { DetailsCard, DetailItemProps } from "@/components/details"
import { removeUnderscore } from "@/lib/utils"

gql`
  mutation sumsubPermalinkCreate($input: SumsubPermalinkCreateInput!) {
    sumsubPermalinkCreate(input: $input) {
      url
    }
  }
`

type ProspectKycStatusProps = {
  prospectId: string
  kycStatus: KycStatus
  level: KycLevel
  applicantId: string | null | undefined
  verificationLink: string | null | undefined
}

export const ProspectKycStatus: React.FC<ProspectKycStatusProps> = ({
  prospectId,
  kycStatus,
  level,
  applicantId,
  verificationLink,
}) => {
  const t = useTranslations("Prospects.ProspectDetails.kycStatus")

  const sumsubLink = `https://cockpit.sumsub.com/checkus#/applicant/${applicantId}/client/basicInfo`

  const [createLink, { loading: linkLoading, error: linkError }] =
    useSumsubPermalinkCreateMutation({
      refetchQueries: ["GetProspectBasicDetails"],
    })

  const handleCreateLink = async () => {
    await createLink({
      variables: {
        input: {
          prospectId,
        },
      },
    })
  }

  const details: DetailItemProps[] = [
    {
      label: t("labels.level"),
      value: removeUnderscore(level),
    },
    {
      label: t("labels.kycApplicationLink"),
      value: applicantId ? (
        <a
          href={sumsubLink}
          target="_blank"
          rel="noopener noreferrer"
          className="text-blue-500 underline"
        >
          {applicantId}
        </a>
      ) : verificationLink ? (
        <div className="flex items-center gap-2">
          <a
            href={verificationLink}
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-500 underline overflow-hidden text-ellipsis whitespace-nowrap max-w-[200px]"
          >
            {verificationLink}
          </a>
          <button
            onClick={() => {
              navigator.clipboard.writeText(verificationLink)
              toast.success(t("messages.copied"))
            }}
          >
            <Copy className="h-4 w-4 cursor-pointer" />
          </button>
        </div>
      ) : (
        <div>
          <button
            onClick={handleCreateLink}
            className="text-blue-500 flex gap-1 items-center"
            disabled={linkLoading}
            data-testid="prospect-create-kyc-link"
          >
            <HiLink />
            {linkLoading ? t("actions.creatingLink") : t("actions.createLink")}
          </button>
          {linkError && <p className="text-red-500">{linkError.message}</p>}
        </div>
      ),
    },
  ]

  return (
    <DetailsCard
      title={t("title")}
      badge={<KycStatusBadge status={kycStatus} />}
      details={details}
      className="w-full md:w-1/4"
      columns={1}
    />
  )
}
