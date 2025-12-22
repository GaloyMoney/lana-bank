"use client"

import React from "react"

import DataTable, { Column } from "@lana/web/components/data-table"

import DateWithTooltip from "@lana/web/components/date-with-tooltip"

import {
  CollateralAction,
  CollateralizationState,
  PendingCreditFacilityCollateralizationState,
  GetCreditFacilityQuery,
} from "@/lib/graphql/generated"

import { cn } from "@/lib/utils"

import Balance from "@/components/balance"

export const formatEntryType = (typename: string) => {
  return typename
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/^\w/, (c) => c.toUpperCase())
}

export const formatCollateralAction = (collateralAction: CollateralAction) => {
  return collateralAction === CollateralAction.Add ? "(Added)" : "(Removed)"
}

const formatEntryTypeWithoutPrefix = (type: string) => {
  const formattedType = formatEntryType(type)
  return formattedType.replace("Credit Facility", "").trim()
}

export const formatCollateralizationState = (
  collateralizationState: CollateralizationState,
) => {
  return collateralizationState
    .toLowerCase()
    .split("_")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ")
}

export const formatPendingCollateralizationState = (
  pendingState: PendingCreditFacilityCollateralizationState,
) => {
  return pendingState
    .toLowerCase()
    .split("_")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ")
}

type CreditFacilityData = NonNullable<GetCreditFacilityQuery["creditFacility"]>

type CreditFacilityHistoryProps = {
  creditFacility: CreditFacilityData
}

type HistoryEntry = CreditFacilityData["history"][number]

export const CreditFacilityHistory: React.FC<CreditFacilityHistoryProps> = ({
  creditFacility,
}) => {
  const columns: Column<HistoryEntry>[] = [
    {
      key: "__typename",
      header: "Entry Type",
      render: (_: HistoryEntry["__typename"], entry: HistoryEntry) => {
        if (!entry.__typename) return "Unknown Entry Type"

        switch (entry.__typename) {
          case "CreditFacilityCollateralUpdated":
            return (
              <div className="flex flex-row gap-1">
                <div>{formatEntryTypeWithoutPrefix(entry.__typename)}</div>
                <div className="text-textColor-secondary text-sm">
                  {formatCollateralAction(entry.action)}
                </div>
              </div>
            )
          case "CreditFacilityCollateralizationUpdated":
            return (
              <div className="flex flex-row gap-1">
                <div>{formatEntryTypeWithoutPrefix(entry.__typename)}</div>
                <div className="text-textColor-secondary text-sm">
                  ({formatCollateralizationState(entry.state)})
                </div>
              </div>
            )
          case "PendingCreditFacilityCollateralizationUpdated":
            return (
              <div className="flex flex-row gap-1">
                <div>{formatEntryTypeWithoutPrefix(entry.__typename)}</div>
                <div className="text-textColor-secondary text-sm">
                  ({formatPendingCollateralizationState(entry.pendingState)})
                </div>
              </div>
            )
          default:
            return formatEntryTypeWithoutPrefix(entry.__typename)
        }
      },
    },
    {
      key: "recordedAt",
      header: "Recorded At",
      render: (recordedAt: string | null | undefined) =>
        recordedAt ? <DateWithTooltip value={recordedAt} /> : "-",
    },
    {
      key: "__typename",
      header: "Amount",
      align: "right",
      render: (_: HistoryEntry["__typename"], entry: HistoryEntry) => {
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
          case "PendingCreditFacilityCollateralizationUpdated":
            return (
              <div className="flex flex-col gap-1 justify-end">
                <Balance amount={entry.collateral} currency="btc" align="end" />
              </div>
            )
          case "CreditFacilityApproved":
          case "CreditFacilityIncrementalPayment":
          case "CreditFacilityDisbursalExecuted":
          case "CreditFacilityInterestAccrued":
          case "CreditFacilityRepaymentAmountReceived":
            return <Balance amount={entry.cents} currency="usd" align="end" />
          case "CreditFacilityCollateralSentOut":
            return <Balance amount={entry.amount} currency="btc" align="end" />
          default:
            return <span>-</span>
        }
      },
    },
  ]

  return (
    <DataTable
      data={creditFacility.history}
      columns={columns}
      emptyMessage={
        <div className="min-h-[10rem] w-full border rounded-md flex items-center justify-center">
          No history found
        </div>
      }
    />
  )
}
