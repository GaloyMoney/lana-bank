"use client"

import React from "react"
import { gql } from "@apollo/client"
import { HiLink } from "react-icons/hi"

import { Copy } from "lucide-react"
import { useTranslations } from "next-intl"
import { toast } from "sonner"

import { KycStatusBadge } from "@/app/customers/kyc-status-badge"

import {
  KycLevel,
  KycVerification,
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

type KycStatusProps = {
  customerId: string
  kycVerification: KycVerification
  level: KycLevel
  applicantId: string | null | undefined
}

export const KycStatus: React.FC<KycStatusProps> = ({
  customerId,
  kycVerification,
  level,
  applicantId,
}) => {
  const t = useTranslations("Customers.CustomerDetails.kycStatus")

  const sumsubLink = `https://cockpit.sumsub.com/checkus#/applicant/${applicantId}/client/basicInfo`

  const [createLink, { data: linkData, loading: linkLoading, error: linkError }] =
    useSumsubPermalinkCreateMutation()

  const handleCreateLink = async () => {
    await createLink({
      variables: {
        input: {
          customerId,
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
              data-testid="customer-create-kyc-link"
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
      badge={<KycStatusBadge status={kycVerification} />}
      details={details}
      className="w-full md:w-1/4"
      columns={1}
    />
  )
}
