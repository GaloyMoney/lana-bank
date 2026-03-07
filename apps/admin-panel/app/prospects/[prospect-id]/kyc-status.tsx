"use client"

import React, { useState, useEffect } from "react"
import { gql } from "@apollo/client"
import { HiLink } from "react-icons/hi"

import { Copy, RotateCcw } from "lucide-react"
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

const ONE_WEEK_MS = 7 * 24 * 60 * 60 * 1000

type ProspectKycStatusProps = {
  prospectId: string
  kycStatus: KycStatus
  level: KycLevel
  applicantId: string | null | undefined
  verificationLink: string | null | undefined
  verificationLinkCreatedAt: string | null | undefined
}

export const ProspectKycStatus: React.FC<ProspectKycStatusProps> = ({
  prospectId,
  kycStatus,
  level,
  applicantId,
  verificationLink,
  verificationLinkCreatedAt,
}) => {
  const t = useTranslations("Prospects.ProspectDetails.kycStatus")

  const sumsubLink = `https://cockpit.sumsub.com/checkus#/applicant/${applicantId}/client/basicInfo`

  const [createLink, { data: linkData, loading: linkLoading, error: linkError }] =
    useSumsubPermalinkCreateMutation({
      refetchQueries: ["GetProspectBasicDetails"],
    })

  const effectiveVerificationLink =
    linkData?.sumsubPermalinkCreate?.url || verificationLink

  const [linkMayBeExpired, setLinkMayBeExpired] = useState(false)
  useEffect(() => {
    if (!verificationLinkCreatedAt) {
      setLinkMayBeExpired(false)
      return
    }
    setLinkMayBeExpired(
      Date.now() - new Date(verificationLinkCreatedAt).getTime() > ONE_WEEK_MS,
    )
  }, [verificationLinkCreatedAt])

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
      ) : effectiveVerificationLink ? (
        <div className="flex flex-col gap-1">
          <div className="flex items-center gap-2">
            <a
              href={effectiveVerificationLink}
              target="_blank"
              rel="noopener noreferrer"
              className="text-blue-500 underline overflow-hidden text-ellipsis whitespace-nowrap max-w-[200px]"
            >
              {effectiveVerificationLink}
            </a>
            <button
              onClick={() => {
                navigator.clipboard.writeText(effectiveVerificationLink)
                toast.success(t("messages.copied"))
              }}
              title={t("actions.copyLink")}
            >
              <Copy className="h-4 w-4 cursor-pointer" />
            </button>
            <button
              onClick={handleCreateLink}
              disabled={linkLoading}
              title={t("actions.refreshLink")}
            >
              <RotateCcw
                className={`h-4 w-4 cursor-pointer ${linkLoading ? "animate-spin" : ""}`}
              />
            </button>
          </div>
          {linkMayBeExpired && (
            <p className="text-amber-500 text-xs">
              {t("messages.linkExpired")}
            </p>
          )}
          {linkError && <p className="text-red-500 text-xs">{linkError.message}</p>}
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
