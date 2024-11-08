"use client"

import React from "react"
import { gql } from "@apollo/client"

import CreditFacilityDetailsCard from "./details"

import { CreditFacilityOverview } from "./overview"

import { CreditFacilityTerms } from "./terms"

import { CreditFacilityDisbursals } from "./disbursals"

import { CreditFacilityTransactions } from "./transactions"

import { useGetCreditFacilityDetailsQuery } from "@/lib/graphql/generated"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/primitive/tab"
import { BreadcrumbLink, BreadCrumbWrapper } from "@/components/breadcrumb-wrapper"
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
        duration {
          period
          units
        }
      }
      disbursals {
        id
        index
        amount
        status
        createdAt
        approvalProcess {
          approvalProcessId
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
      }
      subjectCanUpdateCollateral
      subjectCanInitiateDisbursal
      subjectCanRecordPayment
      subjectCanComplete
    }
  }
`

const CreditFacilityBreadcrumb = ({
  creditFacilityId,
  customerEmail,
}: {
  creditFacilityId: string
  customerEmail: string
}) => {
  const links: BreadcrumbLink[] = [
    { title: "Dashboard", href: "/dashboard" },
    { title: "Credit Facilities", href: "/credit-facilities" },
    {
      title: `${customerEmail} - ${creditFacilityId}`,
      isCurrentPage: true,
    },
  ]

  return <BreadCrumbWrapper links={links} />
}

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
      <CreditFacilityBreadcrumb
        creditFacilityId={data.creditFacility.creditFacilityId}
        customerEmail={data.creditFacility.customer.email}
      />
      <CreditFacilityDetailsCard
        creditFacilityId={creditFacilityId}
        creditFacilityDetails={data.creditFacility}
        refetch={refetch}
      />
      <Tabs defaultValue="overview" className="mt-4">
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
            <CreditFacilityDisbursals
              creditFacility={data.creditFacility}
              refetch={refetch}
            />
          </TabsContent>
        )}
      </Tabs>
    </main>
  )
}

export default CreditFacilityPage
