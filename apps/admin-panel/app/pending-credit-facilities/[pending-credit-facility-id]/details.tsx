"use client"

import React from "react"
import { useTranslations } from "next-intl"
import { Button } from "@lana/web/ui/button"
import { formatDate } from "@lana/web/utils"
import { RefreshCw, ExternalLinkIcon, ArrowRight } from "lucide-react"

import { Label } from "@lana/web/ui/label"

import { toast } from "sonner"

import { PendingCreditFacilityStatusBadge } from "../status-badge"
import { PendingFacilityCollateralizationStateLabel } from "../label"
import { PendingCreditFacilityCollateralUpdateDialog } from "../collateral-update"

import { DetailsCard, DetailItemProps } from "@/components/details"
import Balance from "@/components/balance/balance"
import {
  GetPendingCreditFacilityLayoutDetailsQuery,
  PendingCreditFacilityStatus,
} from "@/lib/graphql/generated"
import { VotersCard } from "@/app/disbursals/[disbursal-id]/voters"

import { removeUnderscore } from "@/lib/utils"
import { mempoolAddressUrl } from "@/app/credit-facilities/[credit-facility-id]/details"
import { usePublicIdForCreditFacility } from "@/hooks/use-public-id"

type PendingCreditFacilityDetailsCardProps = {
  pendingDetails: NonNullable<
    GetPendingCreditFacilityLayoutDetailsQuery["pendingCreditFacility"]
  >
}

const PendingCreditFacilityDetailsCard: React.FC<
  PendingCreditFacilityDetailsCardProps
> = ({ pendingDetails }) => {
  const t = useTranslations("PendingCreditFacilities.PendingDetails.DetailsCard")
  const commonT = useTranslations("Common")

  const [openCollateralUpdateDialog, setOpenCollateralUpdateDialog] =
    React.useState(false)

  const { publicId: facilityPublicId } = usePublicIdForCreditFacility(
    pendingDetails.status === PendingCreditFacilityStatus.Completed
      ? pendingDetails.pendingCreditFacilityId
      : undefined,
  )

  const details: DetailItemProps[] = [
    {
      label: t("details.customer"),
      value: `${pendingDetails.customer.email} (${removeUnderscore(pendingDetails.customer.customerType)})`,
      href: `/customers/${pendingDetails.customer.publicId}`,
    },
    {
      label: t("details.status"),
      value: <PendingCreditFacilityStatusBadge status={pendingDetails.status} />,
    },
    {
      label: t("details.collateralizationState"),
      value: (
        <PendingFacilityCollateralizationStateLabel
          state={pendingDetails.collateralizationState}
        />
      ),
    },
    {
      label: t("details.facilityAmount"),
      value: <Balance amount={pendingDetails.facilityAmount} currency="usd" />,
    },
    {
      label: t("details.createdAt"),
      value: formatDate(pendingDetails.createdAt),
    },
    {
      label: t("details.custodian"),
      value: pendingDetails.wallet?.custodian.name ?? t("details.manual"),
    },
    pendingDetails.wallet?.address && {
      label: (
        <Label className="inline-flex items-center">
          {t("details.walletAddress")}
          <a
            href={mempoolAddressUrl(
              pendingDetails.wallet!.address,
              pendingDetails.wallet!.network,
            )}
            target="_blank"
            className="ml-2 inline-flex items-center gap-1 text-xs text-blue-500 whitespace-nowrap leading-none"
            onClick={(e) => e.stopPropagation()}
          >
            <span className="leading-none">{t("details.viewOnMempool")}</span>
            <ExternalLinkIcon className="h-2.5 w-2.5 shrink-0" aria-hidden="true" />
          </a>
        </Label>
      ),
      value: (
        <span
          onClick={() => {
            navigator.clipboard.writeText(pendingDetails.wallet!.address)
            toast.success(commonT("copiedToClipboard"))
          }}
          className="cursor-pointer hover:bg-secondary font-mono text-sm"
          title={pendingDetails.wallet.address}
        >
          {pendingDetails.wallet.address}
        </span>
      ),
    },
  ].filter(Boolean) as DetailItemProps[]

  const footerContent = (
    <>
      {pendingDetails.status !== PendingCreditFacilityStatus.Completed && (
        <Button
          variant="outline"
          onClick={() => setOpenCollateralUpdateDialog(true)}
          data-testid="update-collateral-button"
        >
          <RefreshCw className="h-4 w-4 mr-2" />
          {t("buttons.updateCollateral")}
        </Button>
      )}
      {pendingDetails.status === PendingCreditFacilityStatus.Completed &&
        facilityPublicId && (
          <Button
            variant="outline"
            data-testid="view-facility-button"
            href={`/credit-facilities/${facilityPublicId}`}
          >
            {t("buttons.viewFacility")}
            <ArrowRight className="h-4 w-4 ml-2" />
          </Button>
        )}
    </>
  )

  return (
    <>
      <DetailsCard
        title={t("title")}
        details={details}
        columns={3}
        footerContent={footerContent}
        errorMessage={pendingDetails.approvalProcess.deniedReason ?? undefined}
      />

      <PendingCreditFacilityCollateralUpdateDialog
        openDialog={openCollateralUpdateDialog}
        setOpenDialog={setOpenCollateralUpdateDialog}
        pendingCreditFacilityId={pendingDetails.pendingCreditFacilityId}
        currentCollateral={pendingDetails.collateral.btcBalance}
        collateralToMatchInitialCvl={pendingDetails.collateralToMatchInitialCvl}
      />

      {pendingDetails.approvalProcess && (
        <VotersCard approvalProcess={pendingDetails.approvalProcess} />
      )}
    </>
  )
}

export default PendingCreditFacilityDetailsCard
