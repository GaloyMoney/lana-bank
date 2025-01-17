"use client"

import React, { useState } from "react"
import { ApolloError, gql } from "@apollo/client"

import { IoCaretDownSharp, IoCaretForwardSharp } from "react-icons/io5"

import { Skeleton } from "@/ui/skeleton"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/ui/table"
import { Tabs, TabsList, TabsContent, TabsTrigger } from "@/ui/tab"
import {
  ChartCategory,
  ChartOfAccountsQuery,
  OffBalanceSheetChartOfAccountsQuery,
  useChartOfAccountsQuery,
  useOffBalanceSheetChartOfAccountsQuery,
} from "@/lib/graphql/generated"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/ui/card"

import { Badge } from "@/ui/badge"

gql`
  fragment ControlSubAccountFields on ChartControlSubAccount {
    name
    accountCode
  }
`

gql`
  fragment ControlAccountFields on ChartControlAccount {
    name
    accountCode
    controlSubAccounts {
      ...ControlSubAccountFields
    }
  }
`

gql`
  fragment CategoryFields on ChartCategory {
    name
    accountCode
    controlAccounts {
      ...ControlAccountFields
    }
  }
`

gql`
  fragment ChartCategories on ChartCategories {
    assets {
      ...CategoryFields
    }
    liabilities {
      ...CategoryFields
    }
    equity {
      ...CategoryFields
    }
    revenues {
      ...CategoryFields
    }
    expenses {
      ...CategoryFields
    }
  }
`

gql`
  query ChartOfAccounts {
    chartOfAccounts {
      name
      categories {
        ...ChartCategories
      }
    }
  }
`

gql`
  query OffBalanceSheetChartOfAccounts {
    offBalanceSheetChartOfAccounts {
      name
      categories {
        ...ChartCategories
      }
    }
  }
`
const AccountCode = ({ code }: { code: string }) => (
  <Badge className="font-mono" variant="secondary">
    {code}
  </Badge>
)

const LoadingSkeleton = () => {
  return (
    <Table data-testid="loading-skeleton">
      <TableHeader>
        <TableRow>
          <TableHead>Account Name</TableHead>
          <TableHead className="text-right">Account Code</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {[1, 2, 3].map((categoryIndex) => (
          <React.Fragment key={`category-${categoryIndex}`}>
            <TableRow>
              <TableCell className="text-primary">
                <Skeleton className="h-6 w-48" />
              </TableCell>
              <TableCell className="text-right">
                <Skeleton className="h-5 w-24 ml-auto" />
              </TableCell>
            </TableRow>
            {[1, 2, 3].map((accountIndex) => (
              <TableRow key={`account-${categoryIndex}-${accountIndex}`}>
                <TableCell className="pl-8">
                  <Skeleton className="h-5 w-64" />
                </TableCell>
                <TableCell className="text-right">
                  <Skeleton className="h-5 w-24 ml-auto" />
                </TableCell>
              </TableRow>
            ))}
          </React.Fragment>
        ))}
      </TableBody>
    </Table>
  )
}

type ChartOfAccountsValuesProps = {
  data:
    | ChartOfAccountsQuery["chartOfAccounts"]
    | OffBalanceSheetChartOfAccountsQuery["offBalanceSheetChartOfAccounts"]
    | undefined
  loading: boolean
  error: ApolloError | undefined
}

const ChartOfAccountsValues = ({ data, loading, error }: ChartOfAccountsValuesProps) => {
  const [expandedAccounts, setExpandedAccounts] = useState<Record<string, boolean>>({})

  if (loading && !data) return <LoadingSkeleton />
  if (error) return <p className="text-destructive">{error.message}</p>
  if (!data?.categories) return null

  const toggleAccount = (accountCode: string) => {
    setExpandedAccounts((prev) => ({
      ...prev,
      [accountCode]: !prev[accountCode],
    }))
  }

  const renderCategory = (category: ChartCategory | null | undefined) => {
    if (!category) return null

    return (
      <React.Fragment key={category.name}>
        <TableRow className="bg-muted/5">
          <TableCell
            data-testid={`category-${category.name.toLowerCase()}`}
            className="text-primary font-bold uppercase tracking-widest leading-8"
          >
            {category.name}
          </TableCell>
          <TableCell className="text-right">
            <AccountCode code={category.accountCode} />
          </TableCell>
        </TableRow>

        {category.controlAccounts.map((control) => (
          <React.Fragment key={control.accountCode}>
            <TableRow
              className={
                control.controlSubAccounts.length > 0
                  ? "cursor-pointer hover:bg-muted/5"
                  : ""
              }
              onClick={() =>
                control.controlSubAccounts.length > 0 &&
                toggleAccount(control.accountCode)
              }
            >
              <TableCell className="pl-8 py-3">
                <div className="flex items-center gap-2">
                  {control.controlSubAccounts.length > 0 && (
                    <span className="text-muted-foreground">
                      {expandedAccounts[control.accountCode] ? (
                        <IoCaretDownSharp className="h-4 w-4" />
                      ) : (
                        <IoCaretForwardSharp className="h-4 w-4" />
                      )}
                    </span>
                  )}
                  <span>{control.name}</span>
                </div>
              </TableCell>
              <TableCell className="text-right">
                <AccountCode code={control.accountCode} />
              </TableCell>
            </TableRow>

            {expandedAccounts[control.accountCode] &&
              control.controlSubAccounts.map((sub) => (
                <TableRow key={sub.accountCode} className="bg-muted/5">
                  <TableCell className="pl-16 py-2">
                    <span className="text-sm">{sub.name}</span>
                  </TableCell>
                  <TableCell className="text-right">
                    <AccountCode code={sub.accountCode} />
                  </TableCell>
                </TableRow>
              ))}
          </React.Fragment>
        ))}
      </React.Fragment>
    )
  }

  return (
    <Table>
      <TableHeader>
        <TableRow className="hover:bg-transparent">
          <TableHead>Account Name</TableHead>
          <TableHead className="text-right">Account Code</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {renderCategory(data.categories.assets)}
        {renderCategory(data.categories.liabilities)}
        {renderCategory(data.categories.equity)}
        {renderCategory(data.categories.revenues)}
        {renderCategory(data.categories.expenses)}
      </TableBody>
    </Table>
  )
}

const ChartOfAccountsPage = () => {
  const {
    data: onBalanceSheetData,
    loading: onBalanceSheetLoading,
    error: onBalanceSheetError,
  } = useChartOfAccountsQuery({
    fetchPolicy: "cache-and-network",
  })

  const {
    data: offBalanceSheetData,
    loading: offBalanceSheetLoading,
    error: offBalanceSheetError,
  } = useOffBalanceSheetChartOfAccountsQuery({
    fetchPolicy: "cache-and-network",
  })

  return (
    <Card>
      <CardHeader>
        <CardTitle>Chart Of Accounts</CardTitle>
        <CardDescription>
          A structured list of all accounts used to categorize and track financial
          records.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Tabs defaultValue="onBalanceSheet">
          <TabsList>
            <TabsTrigger value="onBalanceSheet">Regular</TabsTrigger>
            <TabsTrigger value="offBalanceSheet">Off Balance Sheet</TabsTrigger>
          </TabsList>
          <TabsContent value="onBalanceSheet">
            <ChartOfAccountsValues
              data={onBalanceSheetData?.chartOfAccounts}
              loading={onBalanceSheetLoading}
              error={onBalanceSheetError}
            />
          </TabsContent>
          <TabsContent value="offBalanceSheet">
            <ChartOfAccountsValues
              data={offBalanceSheetData?.offBalanceSheetChartOfAccounts}
              loading={offBalanceSheetLoading}
              error={offBalanceSheetError}
            />
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  )
}

export default ChartOfAccountsPage
