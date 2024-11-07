"use client"

import { gql } from "@apollo/client"

import ActionsList from "../actions/list"

import DashboardCard from "./card"

import {
  useDashboardQuery,
  useGetRealtimePriceUpdatesQuery,
} from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"
import { currencyConverter } from "@/lib/utils"
import { CardSkeleton } from "@/components/card-skeleton"

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
  const { data, loading } = useDashboardQuery({
    fetchPolicy: "no-cache",
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
          {loading ? (
            <>
              <CardSkeleton />
              <CardSkeleton />
              <CardSkeleton />
            </>
          ) : (
            <>
              <DashboardCard
                h1={data?.dashboard.activeFacilities.toString()}
                h2={data?.dashboard.pendingFacilities.toString()}
                h2PopupDescription="Pending Facilities"
                title="Active Facilities"
                description="Credit Facilities where money has been disbursed"
                to="/credit-facilities?filter=active"
              />
              <DashboardCard
                h1={<Balance currency="usd" amount={totalDisbursedUsdCents} />}
                title="Total Disbursed"
                description="Total amount of money customers withdrew from the bank"
                to="/disbursals"
              />
              <DashboardCard
                h1={<Balance currency="usd" amount={totalCollateralUsdCents} />}
                h2={<Balance currency="btc" amount={totalCollateralSats} />}
                title="Total Collateral"
                description="Total bitcoin collateral value at market rate that the bank holds"
                to="/credit-facilities"
              />
            </>
          )}
        </div>
      </div>
      <ActionsList dashboard />
    </div>
  )
}

export default Dashboard
