"use client"
import React from "react"

import { IoCaretForward, IoCaretDown } from "react-icons/io5"

import { PageHeading } from "@/components/page-heading"
import {
  ChartOfAccountsCategoryAccount,
  useGetChartOfAccountsQuery,
} from "@/lib/graphql/generated"

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"

function ChartOfAccountsPage() {
  const [
    displayingSubAccountForCategoryAccounts,
    setDisplayingSubAccountForCategoryAccounts,
  ] = React.useState<Array<string>>([])

  const displayingSubAccountForAny = displayingSubAccountForCategoryAccounts.length > 0
  const isDisplayingSubAccount = (accountId: string) => {
    return displayingSubAccountForCategoryAccounts.includes(accountId)
  }
  const toggleDisplayingSubAccount = (accountId: string) => {
    if (isDisplayingSubAccount(accountId)) {
      setDisplayingSubAccountForCategoryAccounts((prev) =>
        prev.filter((id) => id !== accountId),
      )
    } else {
      setDisplayingSubAccountForCategoryAccounts((prev) => [...prev, accountId])
    }
  }
  const hasSubAccount = (account: ChartOfAccountsCategoryAccount) => {
    return (
      account.__typename === "ChartOfAccountsCategoryAccountSet" && account.hasSubAccounts
    )
  }

  const { data, loading } = useGetChartOfAccountsQuery()

  if (loading) return <div>Loading...</div>

  return (
    <main>
      <PageHeading>{data?.chartOfAccounts?.name}</PageHeading>
      <Table>
        <TableHeader>
          <TableHead>Category</TableHead>
          <TableHead>Account Name</TableHead>
          {displayingSubAccountForAny && <TableHead>Sub-Account Name</TableHead>}
          <TableHead>
            {displayingSubAccountForAny ? "Account or Sub-Account ID" : "Account ID"}
          </TableHead>
        </TableHeader>
        <TableBody>
          {data?.chartOfAccounts?.categories.map((category) => (
            <>
              <TableRow>
                <TableCell className="text-primary font-bold uppercase tracking-widest leading-8">
                  {category.name}
                </TableCell>
                <TableCell />
                {displayingSubAccountForAny && <TableCell />}
                <TableCell />
              </TableRow>
              {category.accounts.map((account, index) => {
                const accountHasSubAccount = hasSubAccount(account)

                return (
                  <>
                    <TableRow
                      onClick={
                        accountHasSubAccount
                          ? () => toggleDisplayingSubAccount(account.id)
                          : undefined
                      }
                      className={accountHasSubAccount ? "hover:cursor-pointer" : ""}
                      key={index}
                    >
                      <TableCell>
                        {hasSubAccount(account) &&
                          (isDisplayingSubAccount(account.id) ? (
                            <IoCaretDown />
                          ) : (
                            <IoCaretForward />
                          ))}
                      </TableCell>
                      <TableCell>{account.name}</TableCell>
                      {displayingSubAccountForAny && <TableCell />}
                      <TableCell className="font-mono">{account.id}</TableCell>
                    </TableRow>
                    {accountHasSubAccount &&
                      isDisplayingSubAccount(account.id) &&
                      account.__typename === "ChartOfAccountsCategoryAccountSet" &&
                      account.subAccounts.map((subAccount) => (
                        <TableRow key={subAccount.id}>
                          <TableCell />
                          <TableCell />
                          <TableCell>{subAccount.name}</TableCell>
                          <TableCell className="font-mono">{subAccount.id}</TableCell>
                        </TableRow>
                      ))}
                  </>
                )
              })}
            </>
          ))}
        </TableBody>
      </Table>
    </main>
  )
}

export default ChartOfAccountsPage
