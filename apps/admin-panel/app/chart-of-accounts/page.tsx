"use client"

import React from "react"
import { gql } from "@apollo/client"

import { Account } from "./accounts"

import { PageHeading } from "@/components/page-heading"
import { Table, TableBody, TableCell, TableRow } from "@/components/primitive/table"
import { Tabs, TabsList, TabsContent, TabsTrigger } from "@/components/primitive/tab"
import {
  GetOffBalanceSheetChartOfAccountsQuery,
  GetOnBalanceSheetChartOfAccountsQuery,
  useGetOffBalanceSheetChartOfAccountsQuery,
  useGetOnBalanceSheetChartOfAccountsQuery,
} from "@/lib/graphql/generated"

gql`
  query GetOnBalanceSheetChartOfAccounts {
    chartOfAccounts {
      name
      categories {
        name
        accounts {
          __typename
          ... on AccountDetails {
            id
            name
          }
          ... on AccountSetDetails {
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
          ... on AccountDetails {
            id
            name
          }
          ... on AccountSetDetails {
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
}
const ChartOfAccountsValues: React.FC<ChartOfAccountsValuesProps> = ({
  data,
  loading,
}) => {
  if (loading) return <p>Loading...</p>

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
                <Account key={account.id} account={account} />
              ))}
            </>
          ))}
      </TableBody>
    </Table>
  )
}

function ChartOfAccountsPage() {
  const { data: onBalanceSheetData, loading: onBalanceSheetLoading } =
    useGetOnBalanceSheetChartOfAccountsQuery()
  const { data: offBalanceSheetData, loading: offBalanceSheetLoading } =
    useGetOffBalanceSheetChartOfAccountsQuery()

  return (
    <main>
      <PageHeading>Chart Of Accounts</PageHeading>
      <Tabs defaultValue="onBalanceSheet">
        <TabsList>
          <TabsTrigger value="onBalanceSheet">Regular</TabsTrigger>
          <TabsTrigger value="offBalanceSheet">Off Balance Sheet</TabsTrigger>
        </TabsList>
        <TabsContent value="onBalanceSheet">
          <ChartOfAccountsValues
            data={onBalanceSheetData?.chartOfAccounts}
            loading={onBalanceSheetLoading}
          />
        </TabsContent>
        <TabsContent value="offBalanceSheet">
          <ChartOfAccountsValues
            data={offBalanceSheetData?.offBalanceSheetChartOfAccounts}
            loading={offBalanceSheetLoading}
          />
        </TabsContent>
      </Tabs>
    </main>
  )
}

export default ChartOfAccountsPage
