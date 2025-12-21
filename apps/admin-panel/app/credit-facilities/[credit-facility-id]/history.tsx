"use client"

import React from "react"
import { useTranslations } from "next-intl"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import {
  CollateralizationStateLabel,
  PendingFacilityCollateralizationStateLabel,
} from "@/app/credit-facilities/label"
import CardWrapper from "@/components/card-wrapper"
import Balance from "@/components/balance/balance"
import { GetCreditFacilityHistoryQuery, CollateralAction } from "@/lib/graphql/generated"
import { formatCollateralAction, cn } from "@/lib/utils"
import DataTable, { Column } from "@/components/data-table"

type CreditFacilityHistoryProps = {
  creditFacility: NonNullable<GetCreditFacilityHistoryQuery["creditFacilityByPublicId"]>
}

type HistoryEntry = CreditFacilityHistoryProps["creditFacility"]["history"][number]

export const CreditFacilityHistory: React.FC<CreditFacilityHistoryProps> = ({
  creditFacility,
}) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.History")

  const columns: Column<HistoryEntry>[] = [
    {
      key: "__typename",
      header: t("columns.entryType"),
      render: (_: HistoryEntry["__typename"], entry: HistoryEntry) => {
        if (!entry.__typename) return t("messages.unknownType")

        switch (entry.__typename) {
          case "CreditFacilityCollateralUpdated":
            return (
              <div className="flex flex-row gap-1">
                <div>{t("entryTypes.collateralUpdated")}</div>
                <div className="text-textColor-secondary text-sm">
                  {formatCollateralAction(entry.action)}
                </div>
              </div>
            )
          case "CreditFacilityCollateralizationUpdated":
            return (
              <div className="flex flex-col gap-1">
                <div>{t("entryTypes.collateralizationUpdated")}</div>
                <div>
                  <CollateralizationStateLabel state={entry.state} />
                </div>
              </div>
            )
          case "PendingCreditFacilityCollateralizationUpdated":
            return (
              <div className="flex flex-col gap-1">
                <div>{t("entryTypes.pendingFacilityCollateralization")}</div>
                <div>
                  <PendingFacilityCollateralizationStateLabel
                    state={entry.pendingState}
                  />
                </div>
              </div>
            )
          case "CreditFacilityApproved":
            return t("entryTypes.approved")
          case "CreditFacilityIncrementalPayment":
            return t("entryTypes.incrementalPayment")
          case "CreditFacilityDisbursalExecuted":
            return t("entryTypes.disbursalExecuted")
          case "CreditFacilityInterestAccrued":
            return t("entryTypes.interestAccrued")
          case "CreditFacilityRepaymentAmountReceived":
            return t("entryTypes.repaymentAmountReceived")
          case "CreditFacilityCollateralSentOut":
            return t("entryTypes.collateralSentOut")
        }
        const exhaustiveCheck: never = entry.__typename
        return exhaustiveCheck
      },
    },
    {
      key: "recordedAt",
      header: t("columns.recordedAt"),
      render: (recordedAt: string | null | undefined) =>
        recordedAt ? <DateWithTooltip value={recordedAt} /> : "-",
    },
    {
      key: "effective",
      header: t("columns.effective"),
      render: (effective: string) => <DateWithTooltip value={effective} />,
    },
    {
      key: "__typename",
      header: t("columns.amount"),
      render: (_: HistoryEntry["__typename"], entry: HistoryEntry) => {
        switch (entry.__typename) {
          case "CreditFacilityCollateralUpdated":
            return (
              <div
                className={cn(
                  "flex gap-1",
                  entry.action === CollateralAction.Add
                    ? "text-success"
                    : "text-destructive",
                )}
              >
                <div>{entry.action === CollateralAction.Add ? "+" : "-"}</div>
                <Balance amount={entry.satoshis} currency="btc" />
              </div>
            )
          case "CreditFacilityCollateralizationUpdated":
          case "PendingCreditFacilityCollateralizationUpdated":
            return <Balance amount={entry.collateral} currency="btc" />
          case "CreditFacilityApproved":
          case "CreditFacilityIncrementalPayment":
          case "CreditFacilityDisbursalExecuted":
          case "CreditFacilityInterestAccrued":
          case "CreditFacilityRepaymentAmountReceived":
            return <Balance amount={entry.cents} currency="usd" />
          case "CreditFacilityCollateralSentOut":
            return <Balance amount={entry.amount} currency="btc" />
          default:
            return <span>-</span>
        }
      },
    },
  ]

  return (
    <CardWrapper title={t("title")} description={t("description")}>
      <DataTable
        data={creditFacility.history}
        columns={columns}
        emptyMessage={t("messages.emptyTable")}
        navigateTo={(entry) =>
          "txId" in entry ? `/ledger-transactions/${entry.txId}` : null
        }
      />
    </CardWrapper>
  )
}
