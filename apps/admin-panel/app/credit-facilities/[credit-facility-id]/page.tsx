"use client"

import React from "react"
import { gql } from "@apollo/client"

import CreditFacilityDetailsCard from "./details"

import { CreditFacilityOverview } from "./overview"

import { CreditFacilityTerms } from "./terms"

import { CreditFacilityDisbursements } from "./disbursements"

import { CreditFacilityTransactions } from "./transactions"

import { PageHeading } from "@/components/page-heading"
import { useGetCreditFacilityDetailsQuery } from "@/lib/graphql/generated"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/primitive/tab"

gql`
  query GetCreditFacilityDetails($id: UUID!) {
    creditFacility(id: $id) {
      id
      approvalProcessId
      creditFacilityId
      collateralizationState
      status
      facilityAmount
      collateral
      createdAt
      expiresAt
      canBeCompleted
      currentCvl {
        total
        disbursed
      }
      collateralToMatchInitialCvl @client
      balance {
        facilityRemaining {
          usdBalance
        }
        disbursed {
          total {
            usdBalance
          }
          outstanding {
            usdBalance
          }
        }
        interest {
          total {
            usdBalance
          }
          outstanding {
            usdBalance
          }
        }
        outstanding {
          usdBalance
        }
        collateral {
          btcBalance
        }
      }
      customer {
        customerId
        email
        telegramId
        status
        level
        applicantId
      }
      creditFacilityTerms {
        annualRate
        accrualInterval
        incurrenceInterval
        liquidationCvl
        marginCallCvl
        initialCvl
        duration {
          period
          units
        }
      }
      disbursements {
        id
        index
        amount
        status
        approvals {
          approvedAt
          user {
            userId
            email
            roles
          }
        }
        createdAt
      }
      transactions {
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
          outstandingDisbursement
          recordedAt
          price
        }
        ... on CreditFacilityDisbursementExecuted {
          cents
          recordedAt
          txId
        }
      }
      userCanUpdateCollateral
      userCanInitiateDisbursement
      userCanApproveDisbursement
      userCanRecordPayment
      userCanComplete
    }
  }
`

function CreditFacilityPage({
  params,
}: {
  params: {
    "credit-facility-id": string
  }
}) {
  const { "credit-facility-id": creditFacilityId } = params
  const { data, loading, error } = useGetCreditFacilityDetailsQuery({
    variables: { id: creditFacilityId },
  })

  if (loading) return <p>Loading...</p>
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.creditFacility) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto">
      <PageHeading>Credit Facility Details</PageHeading>
      <CreditFacilityDetailsCard
        creditFacilityId={creditFacilityId}
        creditFacilityDetails={data.creditFacility}
      />
      <Tabs defaultValue="all" className="mt-4">
        <TabsList>
          <TabsTrigger value="all">All</TabsTrigger>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="terms">Terms</TabsTrigger>
          <TabsTrigger value="transactions">Transactions</TabsTrigger>
          {data.creditFacility.disbursements.length > 0 && (
            <TabsTrigger value="disbursements">Disbursements</TabsTrigger>
          )}
        </TabsList>
        <TabsContent value="all">
          <CreditFacilityOverview creditFacility={data.creditFacility} />
          <CreditFacilityTerms creditFacility={data.creditFacility} />
          {data.creditFacility.disbursements.length > 0 && (
            <CreditFacilityDisbursements creditFacility={data.creditFacility} />
          )}
          <CreditFacilityTransactions creditFacility={data.creditFacility} />
        </TabsContent>
        <TabsContent value="overview">
          <CreditFacilityOverview creditFacility={data.creditFacility} />
        </TabsContent>
        <TabsContent value="terms">
          <CreditFacilityTerms creditFacility={data.creditFacility} />
        </TabsContent>
        <TabsContent value="transactions">
          <CreditFacilityTransactions creditFacility={data.creditFacility} />
        </TabsContent>
        {data.creditFacility.disbursements.length > 0 && (
          <TabsContent value="disbursements">
            <CreditFacilityDisbursements creditFacility={data.creditFacility} />
          </TabsContent>
        )}
      </Tabs>
    </main>
  )
}

export default CreditFacilityPage
