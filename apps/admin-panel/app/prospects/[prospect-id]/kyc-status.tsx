"use client"

import React from "react"
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

type ProspectKycStatusProps = {
  prospectId: string
  kycStatus: KycStatus
  level: KycLevel
  applicantId: string | null | undefined
}

export const ProspectKycStatus: React.FC<ProspectKycStatusProps> = ({
  prospectId,
  kycStatus,
  level,
  applicantId,
}) => {
  const t = useTranslations("Prospects.ProspectDetails.kycStatus")

  const sumsubLink = `https://cockpit.sumsub.com/checkus#/applicant/${applicantId}/client/basicInfo`

  const [createLink, { data: linkData, loading: linkLoading, error: linkError }] =
    useSumsubPermalinkCreateMutation()

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
      ) : (
        <div>
          {!linkData && (
            <button
              onClick={handleCreateLink}
              className="text-blue-500 flex gap-1 items-center"
              disabled={linkLoading}
              data-testid="prospect-create-kyc-link"
            >
              <HiLink />
              {linkLoading ? t("actions.creatingLink") : t("actions.createLink")}
            </button>
          )}
          {linkData && linkData.sumsubPermalinkCreate && (
            <div className="flex items-center gap-2">
              <a
                href={linkData.sumsubPermalinkCreate.url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-500 underline overflow-hidden text-ellipsis whitespace-nowrap max-w-[200px]"
              >
                {linkData.sumsubPermalinkCreate.url}
              </a>
              <button
                onClick={() => {
                  navigator.clipboard.writeText(linkData.sumsubPermalinkCreate.url)
                  toast.success(t("messages.copied"))
                }}
              >
                <Copy className="h-4 w-4 cursor-pointer" />
              </button>
            </div>
          )}
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
