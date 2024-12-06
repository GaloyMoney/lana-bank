"use client"

import React from "react"
import { gql } from "@apollo/client"

import CreditFacilityDetailsCard from "./details"

import { CreditFacilityOverview } from "./overview"

import { CreditFacilityTerms } from "./terms"

import { CreditFacilityDisbursals } from "./disbursals"

import { CreditFacilityTransactions } from "./transactions"

import { useGetCreditFacilityDetailsQuery } from "@/lib/graphql/generated"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/ui/tab"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"

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
      approvalProcess {
        approvalProcessId
        approvalProcessType
        deniedReason
        createdAt
        subjectCanSubmitDecision
        status
        rules {
          ... on CommitteeThreshold {
            threshold
            committee {
              name
              currentMembers {
                email
                roles
              }
            }
          }
          ... on SystemApproval {
            autoApprove
          }
        }
        voters {
          stillEligible
          didVote
          didApprove
          didDeny
          user {
            userId
            email
            roles
          }
        }
      }
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
        oneTimeFeeRate
        duration {
          period
          units
        }
      }
      disbursals {
        id
        disbursalId
        index
        amount
        status
        createdAt
        approvalProcess {
          approvalProcessId
          deniedReason
          approvalProcessType
          createdAt
          subjectCanSubmitDecision
          status
          rules {
            ... on CommitteeThreshold {
              threshold
              committee {
                name
                currentMembers {
                  email
                  roles
                }
              }
            }
            ... on SystemApproval {
              autoApprove
            }
          }
          voters {
            stillEligible
            didVote
            didApprove
            didDeny
            user {
              userId
              email
              roles
            }
          }
        }
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
      subjectCanUpdateCollateral
      subjectCanInitiateDisbursal
      subjectCanRecordPayment
      subjectCanComplete
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
  const { data, loading, error, refetch } = useGetCreditFacilityDetailsQuery({
    variables: { id: creditFacilityId },
  })

  if (loading) return <DetailsPageSkeleton detailItems={4} tabs={3} tabsCards={1} />
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.creditFacility) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto">
      <CreditFacilityDetailsCard
        creditFacilityId={creditFacilityId}
        creditFacilityDetails={data.creditFacility}
        refetch={refetch}
      />
      <Tabs defaultValue="overview" className="mt-2">
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="terms">Terms</TabsTrigger>
          <TabsTrigger value="transactions">Transactions</TabsTrigger>
          {data.creditFacility.disbursals.length > 0 && (
            <TabsTrigger value="disbursals">Disbursals</TabsTrigger>
          )}
        </TabsList>
        <TabsContent value="overview">
          <CreditFacilityOverview creditFacility={data.creditFacility} />
        </TabsContent>
        <TabsContent value="terms">
          <CreditFacilityTerms creditFacility={data.creditFacility} />
        </TabsContent>
        <TabsContent value="transactions">
          <CreditFacilityTransactions creditFacility={data.creditFacility} />
        </TabsContent>
        {data.creditFacility.disbursals.length > 0 && (
          <TabsContent value="disbursals">
            <CreditFacilityDisbursals creditFacility={data.creditFacility} />
          </TabsContent>
        )}
      </Tabs>
    </main>
  )
}

export default CreditFacilityPage
