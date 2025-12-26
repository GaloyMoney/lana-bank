"use client"

import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import ActionsList from "../actions/list"

import DashboardCard from "./dashboard-card"

import {
  useDashboardQuery,
  useGetRealtimePriceUpdatesQuery,
} from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"
import { currencyConverter } from "@/lib/utils"
import { CardSkeleton } from "@/components/card-skeleton"
import { UsdCents } from "@/types"

gql`
  query Dashboard {
    dashboard {
      activeFacilities
      pendingFacilities
      totalDisbursed
      totalCollateral
    }
  }
`

const Dashboard = () => {
  const t = useTranslations("Dashboard")

  const { data, loading } = useDashboardQuery({
    fetchPolicy: "cache-and-network",
  })
  const { data: priceData } = useGetRealtimePriceUpdatesQuery({
    fetchPolicy: "cache-only",
  })

  const totalCollateralSats = data?.dashboard.totalCollateral
  const totalDisbursedUsdCents = data?.dashboard.totalDisbursed

  const totalCollateralUsdCents =
    priceData?.realtimePrice?.usdCentsPerBtc && totalCollateralSats
      ? currencyConverter.satoshiToBtc(
          priceData.realtimePrice.usdCentsPerBtc * totalCollateralSats,
        )
      : 0

  return (
    <div className="w-full h-full flex flex-col gap-2">
      <div className="relative w-full flex flex-col justify-center items-start">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-2 w-full">
          {loading && !data ? (
            <>
              <CardSkeleton />
              <CardSkeleton />
              <CardSkeleton />
            </>
          ) : (
            <>
              <DashboardCard
                h1={data?.dashboard.activeFacilities.toString()}
                h2={
                  data?.dashboard.pendingFacilities.toString() +
                  " " +
                  t("cards.activeFacilities.pending")
                }
                title={t("cards.activeFacilities.title")}
                description={t("cards.activeFacilities.description")}
                to="/credit-facilities?filter=active"
                buttonText={t("cards.activeFacilities.buttonText")}
              />
              {totalDisbursedUsdCents !== undefined && (
                <DashboardCard
                  h1={<Balance currency="usd" amount={totalDisbursedUsdCents} />}
                  title={t("cards.totalDisbursed.title")}
                  description={t("cards.totalDisbursed.description")}
                  to="/disbursals"
                  buttonText={t("cards.totalDisbursed.buttonText")}
                />
              )}
              {totalCollateralSats !== undefined && (
                <DashboardCard
                  h1={
                    <Balance
                      currency="usd"
                      amount={totalCollateralUsdCents as UsdCents}
                    />
                  }
                  h2={<Balance currency="btc" amount={totalCollateralSats} />}
                  title={t("cards.totalCollateral.title")}
                  description={t("cards.totalCollateral.description")}
                  to="/credit-facilities"
                  buttonText={t("cards.totalCollateral.buttonText")}
                />
              )}
            </>
          )}
        </div>
      </div>
      <ActionsList dashboard />
    </div>
  )
}

export default Dashboard
