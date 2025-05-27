"use client"

import React from "react"
import { useTranslations } from "next-intl"

import { CollateralizationStateLabel } from "@/app/credit-facilities/label"
import CardWrapper from "@/components/card-wrapper"
import Balance from "@/components/balance/balance"
import {
  GetCreditFacilityHistoryQuery,
  CreditFacilityHistoryEntry,
  CollateralAction,
} from "@/lib/graphql/generated"
import { formatCollateralAction, cn } from "@/lib/utils"
import DataTable, { Column } from "@/components/data-table"
import DateWithTooltip from "@/components/date-with-tooltip"

type CreditFacilityHistoryProps = {
  creditFacility: NonNullable<GetCreditFacilityHistoryQuery["creditFacility"]>
}

export const CreditFacilityHistory: React.FC<CreditFacilityHistoryProps> = ({
  creditFacility,
}) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.History")

  const columns: Column<CreditFacilityHistoryEntry>[] = [
    {
      key: "__typename",
      header: t("columns.entryType"),
      render: (
        _: CreditFacilityHistoryEntry["__typename"],
        entry: CreditFacilityHistoryEntry,
      ) => {
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
              <div className="flex flex-row gap-1">
                <div>{t("entryTypes.collateralizationUpdated")}</div>
                <div className="text-textColor-secondary text-sm">
                  (<CollateralizationStateLabel state={entry.state} />)
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
      align: "right",
      render: (
        _: CreditFacilityHistoryEntry["__typename"],
        entry: CreditFacilityHistoryEntry,
      ) => {
        switch (entry.__typename) {
          case "CreditFacilityCollateralUpdated":
            return (
              <div
                className={cn(
                  "flex justify-end gap-1",
                  entry.action === CollateralAction.Add
                    ? "text-success"
                    : "text-destructive",
                )}
              >
                <div>{entry.action === CollateralAction.Add ? "+" : "-"}</div>
                <Balance amount={entry.satoshis} currency="btc" align="end" />
              </div>
            )
          case "CreditFacilityCollateralizationUpdated":
            return (
              <div className="flex flex-col gap-1 justify-end">
                <Balance amount={entry.collateral} currency="btc" align="end" />
              </div>
            )
          case "CreditFacilityApproved":
          case "CreditFacilityIncrementalPayment":
          case "CreditFacilityDisbursalExecuted":
          case "CreditFacilityInterestAccrued":
            return <Balance amount={entry.cents} currency="usd" align="end" />
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
      />
    </CardWrapper>
  )
}
