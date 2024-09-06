"use client"

import { useEffect } from "react"
import { gql } from "@apollo/client"

import { LoanSnapshot } from "./snapshot"
import { LoanDetailsCard } from "./details"
import { LoanTerms } from "./terms"
import { LoanTransactionHistory } from "./transactions"
import { LoanApprovers } from "./approvers"

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/primitive/tab"
import { PageHeading } from "@/components/page-heading"

import {
  useGetLoanDetailsQuery,
  useGetRealtimePriceUpdatesQuery,
} from "@/lib/graphql/generated"

gql`
  query GetLoanDetails($id: UUID!) {
    loan(id: $id) {
      id
      loanId
      createdAt
      approvedAt
      principal
      expiresAt
      collateral
      status
      collateralizationState
      customer {
        customerId
        email
      }
      balance {
        collateral {
          btcBalance
        }
        outstanding {
          usdBalance
        }
        interestIncurred {
          usdBalance
        }
      }
      transactions {
        ... on IncrementalPayment {
          cents
          recordedAt
          txId
        }
        ... on InterestAccrued {
          cents
          recordedAt
          txId
        }
        ... on CollateralUpdated {
          satoshis
          recordedAt
          action
          txId
        }
        ... on LoanOrigination {
          cents
          recordedAt
          txId
        }
        ... on CollateralizationUpdated {
          state
          outstandingPrincipal
          outstandingInterest
          price
          collateral
          recordedAt
        }
      }
      loanTerms {
        annualRate
        interval
        liquidationCvl
        marginCallCvl
        initialCvl
        duration {
          period
          units
        }
      }
      approvals {
        user {
          email
          roles
        }
        approvedAt
      }
      currentCvl @client
      collateralToMatchInitialCvl @client
    }
  }
`

const Loan = ({
  params,
}: {
  params: {
    "loan-id": string
  }
}) => {
  const { "loan-id": loanId } = params

  const { data, loading, error, refetch } = useGetLoanDetailsQuery({
    variables: { id: loanId },
  })

  const { data: priceInfo } = useGetRealtimePriceUpdatesQuery({
    fetchPolicy: "cache-only",
  })

  // If price changes, refetch current CVL
  useEffect(() => {
    refetch()
  }, [priceInfo?.realtimePrice.usdCentsPerBtc, refetch])

  const hasApprovers = !!data?.loan?.approvals?.length

  return (
    <main className="max-w-7xl m-auto">
      <PageHeading>Loan Details</PageHeading>
      {loading && <p>Loading...</p>}
      {error && <div className="text-destructive">{error.message}</div>}
      {data && data.loan && (
        <>
          <LoanDetailsCard loan={data.loan} refetch={refetch} />
          <Tabs defaultValue="overview" className="mt-4">
            <TabsList>
              <TabsTrigger value="overview">Overview</TabsTrigger>
              <TabsTrigger value="snapshot">Snapshot</TabsTrigger>
              <TabsTrigger value="transactions">Transactions</TabsTrigger>
              <TabsTrigger value="terms">Terms</TabsTrigger>
              {hasApprovers && <TabsTrigger value="approvers">Approvers</TabsTrigger>}
            </TabsList>
            <TabsContent value="overview">
              <LoanSnapshot loan={data.loan} />
              <LoanTerms loan={data.loan} />
              <LoanTransactionHistory loan={data.loan} />
            </TabsContent>
            <TabsContent value="snapshot">
              <LoanSnapshot loan={data.loan} />
            </TabsContent>
            <TabsContent value="transactions">
              <LoanTransactionHistory loan={data.loan} />
            </TabsContent>
            <TabsContent value="terms">
              <LoanTerms loan={data.loan} />
            </TabsContent>
            {hasApprovers && (
              <TabsContent value="approvers">
                <LoanApprovers loan={data.loan} />
              </TabsContent>
            )}
          </Tabs>
        </>
      )}
    </main>
  )
}

export default Loan
