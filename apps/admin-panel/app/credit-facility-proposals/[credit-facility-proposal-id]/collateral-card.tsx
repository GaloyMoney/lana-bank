import React from "react"
import { useTranslations } from "next-intl"

import Balance from "@/components/balance/balance"
import { DetailsCard, DetailItemProps } from "@/components/details"
import {
  GetCreditFacilityProposalLayoutDetailsQuery,
  useGetRealtimePriceUpdatesQuery,
} from "@/lib/graphql/generated"
import { CENTS_PER_USD, formatCvl, SATS_PER_BTC } from "@/lib/utils"
import { Satoshis, UsdCents } from "@/types"

type CreditFacilityProposalCollateralProps = {
  proposal: NonNullable<
    GetCreditFacilityProposalLayoutDetailsQuery["creditFacilityProposal"]
  >
}

export const CreditFacilityProposalCollateral: React.FC<
  CreditFacilityProposalCollateralProps
> = ({ proposal }) => {
  const t = useTranslations("CreditFacilityProposals.ProposalDetails.CollateralCard")

  const { data: priceInfo } = useGetRealtimePriceUpdatesQuery({
    fetchPolicy: "cache-only",
  })

  const collateralInUsd = priceInfo
    ? (proposal.collateral.btcBalance / SATS_PER_BTC) *
      (priceInfo.realtimePrice.usdCentsPerBtc / CENTS_PER_USD)
    : 0

  const collateralDependentDetails: DetailItemProps[] = [
    {
      label: t("details.collateralBalance"),
      value: <Balance amount={proposal.collateral.btcBalance} currency="btc" />,
    },
    {
      label: t("details.currentPrice"),
      value: priceInfo && (
        <Balance amount={priceInfo.realtimePrice.usdCentsPerBtc} currency="usd" />
      ),
    },
    {
      label: t("details.collateralValue"),
      value: priceInfo && (
        <Balance amount={(collateralInUsd * CENTS_PER_USD) as UsdCents} currency="usd" />
      ),
    },
    {
      label: t("details.collateralToReachTarget", {
        percentage: formatCvl(proposal.creditFacilityTerms.initialCvl),
      }),
      value: (
        <Balance
          amount={(proposal.collateralToMatchInitialCvl ?? 0) as Satoshis}
          currency="btc"
        />
      ),
      valueTestId: "collateral-to-reach-target",
    },
  ]

  return (
    <DetailsCard
      className="w-[45%]"
      title={t("title")}
      details={collateralDependentDetails}
      columns={2}
    />
  )
}
