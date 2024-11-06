"use client"

import { gql } from "@apollo/client"

import React from "react"

import { CustomerDetailsCard } from "./details"
import { CustomerAccountBalances } from "./balances"
import { CustomerTransactionsTable } from "./transactions"
import { KycStatus } from "./kyc-status"
import { Documents } from "./documents"

import { CustomerCreditFacilitiesTable } from "./credit-facilities"

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/primitive/tab"
import { PageHeading } from "@/components/page-heading"
import { useGetCustomerQuery } from "@/lib/graphql/generated"

gql`
  query GetCustomer($id: UUID!) {
    customer(id: $id) {
      customerId
      email
      telegramId
      status
      level
      applicantId
      subjectCanRecordDeposit
      subjectCanInitiateWithdrawal
      subjectCanCreateCreditFacility
      balance {
        checking {
          settled
          pending
        }
      }
      creditFacilities {
        id
        creditFacilityId
        collateralizationState
        status
        balance {
          collateral {
            btcBalance
          }
          outstanding {
            usdBalance
          }
        }
      }
      deposits {
        createdAt
        customerId
        depositId
        reference
        amount
      }
      withdrawals {
        status
        reference
        customerId
        createdAt
        withdrawalId
        amount
        customer {
          customerId
          email
        }
      }
      transactions @client {
        ... on Deposit {
          createdAt
          customerId
          depositId
          reference
          amount
        }
        ... on Withdrawal {
          status
          reference
          customerId
          withdrawalId
          createdAt
          amount
          customer {
            customerId
            email
          }
        }
      }
      documents {
        id
        filename
      }
    }
  }
`

const Customer = ({
  params,
}: {
  params: {
    "customer-id": string
  }
}) => {
  const { "customer-id": customerId } = params

  const { data, loading, error, refetch } = useGetCustomerQuery({
    variables: { id: customerId },
  })

  return (
    <main className="max-w-7xl m-auto">
      <PageHeading>Customer Details</PageHeading>
      {loading && <p>Loading...</p>}
      {error && <div className="text-destructive">{error.message}</div>}
      {data && data.customer && (
        <>
          <CustomerDetailsCard customer={data.customer} refetch={refetch} />
          <Tabs defaultValue="overview" className="mt-4">
            <TabsList>
              <TabsTrigger value="overview">Overview</TabsTrigger>
              <TabsTrigger value="balances">Balances</TabsTrigger>
              <TabsTrigger value="credit-facilities">Credit Facilities</TabsTrigger>
              <TabsTrigger value="transactions">Transactions</TabsTrigger>
              <TabsTrigger value="kyc">KYC Status</TabsTrigger>
              <TabsTrigger value="docs">Documents</TabsTrigger>
            </TabsList>
            <TabsContent value="overview">
              <CustomerAccountBalances balance={data.customer.balance} />
              <CustomerTransactionsTable transactions={data.customer.transactions} />
              <KycStatus customerId={customerId} />
            </TabsContent>
            <TabsContent value="balances">
              <CustomerAccountBalances balance={data.customer.balance} />
            </TabsContent>
            <TabsContent value="credit-facilities">
              <CustomerCreditFacilitiesTable
                creditFacilities={data.customer.creditFacilities}
              />
            </TabsContent>
            <TabsContent value="transactions">
              <CustomerTransactionsTable transactions={data.customer.transactions} />
            </TabsContent>
            <TabsContent value="kyc">
              <KycStatus customerId={customerId} />
            </TabsContent>
            <TabsContent value="docs">
              <Documents customer={data.customer} refetch={refetch} />
            </TabsContent>
          </Tabs>
        </>
      )}
    </main>
  )
}

export default Customer
