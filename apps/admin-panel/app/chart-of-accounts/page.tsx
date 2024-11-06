"use client"

import React, { useState } from "react"
import { ApolloError, gql } from "@apollo/client"

import { Account } from "./accounts"

import { PageHeading } from "@/components/page-heading"
import { Table, TableBody, TableCell, TableRow } from "@/components/primitive/table"
import { Tabs, TabsList, TabsContent, TabsTrigger } from "@/components/primitive/tab"
import {
  AccountAmountsByCurrency,
  GetOffBalanceSheetChartOfAccountsQuery,
  GetOnBalanceSheetChartOfAccountsQuery,
  useGetOffBalanceSheetChartOfAccountsQuery,
  useGetOnBalanceSheetChartOfAccountsQuery,
} from "@/lib/graphql/generated"
import { DateRange, getInitialDateRange } from "@/components/date-range-picker"

gql`
  query GetOnBalanceSheetChartOfAccounts {
    chartOfAccounts {
      name
      categories {
        name
        accounts {
          __typename
          ... on Account {
            id
            name
          }
          ... on AccountSet {
            id
            name
            hasSubAccounts
          }
        }
      }
    }
  }

  query GetOffBalanceSheetChartOfAccounts {
    offBalanceSheetChartOfAccounts {
      name
      categories {
        name
        accounts {
          __typename
          ... on Account {
            id
            name
          }
          ... on AccountSet {
            id
            name
            hasSubAccounts
          }
        }
      }
    }
  }
`

type ChartOfAccountsValuesProps = {
  data:
    | GetOnBalanceSheetChartOfAccountsQuery["chartOfAccounts"]
    | GetOffBalanceSheetChartOfAccountsQuery["offBalanceSheetChartOfAccounts"]
    | undefined
  loading: boolean
  error: ApolloError | undefined
  dateRange: DateRange
}
const ChartOfAccountsValues: React.FC<ChartOfAccountsValuesProps> = ({
  data,
  loading,
  error,
  dateRange,
}) => {
  if (loading) return <p>Loading...</p>
  if (error) return <p className="text-destructive">{error.message}</p>

  return (
    <Table>
      <TableBody>
        {data?.categories
          // without the sort, the categories are being displayed randomly
          .toSorted(({ name: str1 }, { name: str2 }) =>
            str1 < str2 ? -1 : +(str1 > str2),
          )
          .map((category) => (
            <>
              <TableRow>
                <TableCell className="text-primary font-bold uppercase tracking-widest leading-8">
                  {category.name}
                </TableCell>
              </TableRow>
              {category.accounts.map((account) => (
                <Account
                  key={account.id}
                  dateRange={dateRange}
                  account={{
                    ...account,
                    amounts: undefined as unknown as AccountAmountsByCurrency,
                  }}
                />
              ))}
            </>
          ))}
      </TableBody>
    </Table>
  )
}

function ChartOfAccountsPage() {
  const {
    data: onBalanceSheetData,
    loading: onBalanceSheetLoading,
    error: onBalanceSheetError,
  } = useGetOnBalanceSheetChartOfAccountsQuery()
  const {
    data: offBalanceSheetData,
    loading: offBalanceSheetLoading,
    error: offBalanceSheetError,
  } = useGetOffBalanceSheetChartOfAccountsQuery()

  const [dateRange] = useState<DateRange>(getInitialDateRange)

  return (
    <main>
      <PageHeading>Chart Of Accounts</PageHeading>
      <Tabs defaultValue="onBalanceSheet" className="mt-4">
        <TabsList>
          <TabsTrigger value="onBalanceSheet">Regular</TabsTrigger>
          <TabsTrigger value="offBalanceSheet">Off Balance Sheet</TabsTrigger>
        </TabsList>
        <TabsContent value="onBalanceSheet">
          <ChartOfAccountsValues
            data={onBalanceSheetData?.chartOfAccounts}
            loading={onBalanceSheetLoading}
            error={onBalanceSheetError}
            dateRange={dateRange}
          />
        </TabsContent>
        <TabsContent value="offBalanceSheet">
          <ChartOfAccountsValues
            data={offBalanceSheetData?.offBalanceSheetChartOfAccounts}
            loading={offBalanceSheetLoading}
            error={offBalanceSheetError}
            dateRange={dateRange}
          />
        </TabsContent>
      </Tabs>
    </main>
  )
}

export default ChartOfAccountsPage
