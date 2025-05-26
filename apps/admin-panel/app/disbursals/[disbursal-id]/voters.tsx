"use client"
import { FaBan, FaCheckCircle, FaQuestion } from "react-icons/fa"
import { useTranslations } from "next-intl"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import {
  ApprovalProcessStatus,
  GetDisbursalDetailsQuery,
  GetWithdrawalDetailsQuery,
} from "@/lib/graphql/generated"

export const VotersCard = ({
  approvalProcess,
}: {
  approvalProcess:
    | NonNullable<GetDisbursalDetailsQuery["disbursal"]>["approvalProcess"]
    | NonNullable<GetWithdrawalDetailsQuery["withdrawal"]>["approvalProcess"]
    | null
}) => {
  const t = useTranslations("Disbursals.DisbursalDetails.VotersCard")

  if (!approvalProcess) {
    return null
  }

  if (approvalProcess?.rules.__typename !== "CommitteeThreshold") {
    return null
  }

  return (
    <Card className="mt-2">
      <CardHeader>
        <CardTitle>
          {t("title", { committeeName: approvalProcess.rules.committee?.name })}
        </CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent>
        {approvalProcess.voters
          .filter((voter) => {
            if (
              approvalProcess.status === ApprovalProcessStatus.InProgress ||
              ([ApprovalProcessStatus.Approved, ApprovalProcessStatus.Denied].includes(
                approvalProcess.status as ApprovalProcessStatus,
              ) &&
                voter.didVote)
            ) {
              return true
            }
            return false
          })
          .map((voter) => (
            <div key={voter.user.userId} className="flex items-center space-x-3 p-2">
              {voter.didApprove ? (
                <FaCheckCircle className="h-6 w-6 text-green-500" />
              ) : voter.didDeny ? (
                <FaBan className="h-6 w-6 text-red-500" />
              ) : !voter.didVote ? (
                <FaQuestion className="h-6 w-6 text-textColor-secondary" />
              ) : (
                <>{/* Impossible */}</>
              )}
              <div>
                <p className="text-sm font-medium">{voter.user.email}</p>
                <p className="text-sm text-textColor-secondary">
                  {voter.user.role?.name}
                </p>
                {
                  <p className="text-xs text-textColor-secondary">
                    {voter.didApprove && t("voter.approved")}
                    {voter.didDeny && t("voter.denied")}
                    {!voter.didVote && t("voter.notVoted")}
                  </p>
                }
              </div>
            </div>
          ))}
      </CardContent>
    </Card>
  )
}
