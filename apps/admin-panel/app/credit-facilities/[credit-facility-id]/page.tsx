"use client"

import { gql } from "@apollo/client"
import { use } from "react"
import { useTranslations } from "next-intl"

import { CreditFacilityHistory } from "./history"

import Balance from "@/components/balance/balance"

import {
  useGetCreditFacilityHistoryQuery,
  useGetCustomerBasicDetailsQuery,
} from "@/lib/graphql/generated"
import { removeUnderscore } from "@/lib/utils"

gql`
  fragment CreditFacilityHistoryFragment on CreditFacility {
    id
    creditFacilityId
    createdAt
    maturesAt
    customer {
      customerId
    }
    history {
      ... on CreditFacilityIncrementalPayment {
        cents
        recordedAt
        txId
      }
      ... on CreditFacilityCollateralUpdated {
        satoshis
        recordedAt
        action
        txId
      }
      ... on CreditFacilityOrigination {
        cents
        recordedAt
        txId
      }
      ... on CreditFacilityCollateralizationUpdated {
        state
        collateral
        outstandingInterest
        outstandingDisbursal
        recordedAt
        price
      }
      ... on CreditFacilityDisbursalExecuted {
        cents
        recordedAt
        txId
      }
      ... on CreditFacilityInterestAccrued {
        cents
        recordedAt
        txId
        days
      }
    }
    repaymentPlan {
      initial
      status
    }
  }

  query GetCreditFacilityHistory($id: UUID!) {
    creditFacility(id: $id) {
      ...CreditFacilityHistoryFragment
    }
  }
`

interface CreditFacilityHistoryPageProps {
  params: Promise<{
    "credit-facility-id": string
  }>
}

export default function CreditFacilityHistoryPage({
  params,
}: CreditFacilityHistoryPageProps) {
  const { "credit-facility-id": creditFacilityId } = use(params)
  const { data: cfData } = useGetCreditFacilityHistoryQuery({
    variables: { id: creditFacilityId },
    fetchPolicy: "cache-and-network",
  })

  const customerId = cfData?.creditFacility?.customer?.customerId
  const { data: customerData } = useGetCustomerBasicDetailsQuery({
    variables: { id: customerId! },
    skip: !customerId,
  })

  const t = useTranslations()

  if (!cfData?.creditFacility) return null

  const customerType = customerData?.customer?.customerType
  const customerTypeDisplay = customerType ? removeUnderscore(customerType) : "Unknown"

  // Use type assertion to tell TypeScript that createdAt exists
  const creditFacility = cfData.creditFacility as unknown as {
    id: string
    creditFacilityId: string
    createdAt: string
    customer: { customerId: string }
    history: any[]
    repaymentPlan: any[]
  }

  const issuanceDate = new Date(creditFacility.createdAt).toLocaleDateString()

  const monthlyPayment = (creditFacility.repaymentPlan
    ?.filter(
      (payment: { status: string }) =>
        payment.status === "UPCOMING" || payment.status === "NOT_YET_DUE",
    )
    .reduce((acc: number, payment: { initial: number }) => acc + payment.initial, 0) /
    12) as CurrencyType

  return (
    <div>
      <div className="space-y-2 mb-4">
        <div>
          <span className="font-medium">{t("Common.customerType")}: </span>
          <span>{customerTypeDisplay}</span>
        </div>
        <div>
          <span className="font-medium">{t("Common.dateOfIssuance")}: </span>
          <span>{issuanceDate}</span>
        </div>
        <div>
          <span className="font-medium">{t("table.headers.monthlyPayment")}: </span>
          <Balance
            align="end"
            className="font-semibold"
            currency="usd"
            amount={monthlyPayment || 0}
          />
        </div>
      </div>
      <CreditFacilityHistory creditFacility={creditFacility} />
    </div>
  )
}
