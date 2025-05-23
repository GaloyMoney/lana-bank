"use client"

import { gql } from "@apollo/client"
import { use } from "react"
import { useTranslations } from "next-intl"

import { CreditFacilityHistory } from "./history"

import Balance from "@/components/balance/balance"

import {
  useGetCreditFacilityHistoryQuery,
  useGetCreditFacilityHistoryDetailsQuery,
  useGetCustomerBasicDetailsQuery,
} from "@/lib/graphql/generated"
import { removeUnderscore } from "@/lib/utils"

gql`
  fragment CreditFacilityHistoryFragment on CreditFacility {
    id
    creditFacilityId
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

  query GetCreditFacilityHistoryDetails($id: UUID!) {
    creditFacility(id: $id) {
      id
      createdAt
      maturesAt
      customer {
        customerId
      }
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

  const { data: layoutData } = useGetCreditFacilityHistoryDetailsQuery({
    variables: { id: creditFacilityId },
  })

  const customerId = layoutData?.creditFacility?.customer?.customerId
  const { data: customerData } = useGetCustomerBasicDetailsQuery({
    variables: { id: customerId! },
    skip: !customerId,
  })

  const t = useTranslations()

  if (!cfData?.creditFacility || !layoutData?.creditFacility) return null

  const customerType = customerData?.customer?.customerType
  const customerTypeDisplay = customerType ? removeUnderscore(customerType) : "Unknown"

  const issuanceDate = new Date(layoutData.creditFacility.createdAt).toLocaleDateString()

  const monthlyPayment = (cfData.creditFacility.repaymentPlan
    ?.filter(
      (payment) => payment.status === "UPCOMING" || payment.status === "NOT_YET_DUE",
    )
    .reduce((acc, payment) => acc + payment.initial, 0) / 12) as CurrencyType

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
      <CreditFacilityHistory creditFacility={cfData.creditFacility} />
    </div>
  )
}
