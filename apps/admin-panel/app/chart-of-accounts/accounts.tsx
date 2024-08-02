"use client"

import React from "react"
import { gql } from "@apollo/client"
import { IoCaretDownSharp, IoCaretForwardSharp } from "react-icons/io5"

import {
  AccountBalancesByCurrency,
  AccountSetSubAccountWithBalance,
  useChartOfAccountsAccountSetQuery,
} from "@/lib/graphql/generated"
import { TableCell, TableRow } from "@/components/primitive/table"

gql`
  query ChartOfAccountsAccountSet($accountSetId: UUID!, $first: Int!, $after: String) {
    accountSetWithBalance(accountSetId: $accountSetId) {
      id
      name
      subAccounts(first: $first, after: $after) {
        edges {
          cursor
          node {
            __typename
            ... on AccountWithBalance {
              __typename
              id
              name
            }
            ... on AccountSetWithBalance {
              __typename
              id
              name
              hasSubAccounts
            }
          }
        }
        pageInfo {
          hasNextPage
        }
      }
    }
  }
`

type AccountProps = {
  depth?: number
  account: AccountSetSubAccountWithBalance
}

const SubAccountsForAccountSet: React.FC<AccountProps> = ({ account, depth = 0 }) => {
  const { data, fetchMore } = useChartOfAccountsAccountSetQuery({
    variables: {
      accountSetId: account.id,
      first: 2,
    },
  })

  const hasMoreSubAccounts = data?.accountSetWithBalance?.subAccounts.pageInfo.hasNextPage
  const subAccounts = data?.accountSetWithBalance?.subAccounts.edges

  return (
    <>
      {subAccounts?.map((subAccount) => (
        <Account
          key={subAccount.node.id}
          account={{
            ...subAccount.node,
            balance: undefined as unknown as AccountBalancesByCurrency,
          }}
          depth={depth + 1}
        />
      ))}
      {hasMoreSubAccounts && subAccounts && (
        <TableRow>
          <TableCell
            className="flex items-center cursor-pointer"
            onClick={() =>
              fetchMore({
                variables: {
                  after: subAccounts[subAccounts.length - 1].cursor,
                },
              })
            }
          >
            {Array.from({ length: depth + 1 }).map((_, i) => (
              <div key={i} className="w-8" />
            ))}
            <div className="w-8" />
            <div className="font-thin italic">show more...</div>
          </TableCell>
        </TableRow>
      )}
    </>
  )
}

export const Account: React.FC<AccountProps> = ({ account, depth = 0 }) => {
  const [showingSubAccounts, setShowingSubAccounts] = React.useState(false)
  const hasSubAccounts =
    account.__typename === "AccountSetWithBalance" && account.hasSubAccounts

  return (
    <>
      <TableRow
        key={account.id}
        className={hasSubAccounts ? "cursor-pointer" : ""}
        onClick={() => setShowingSubAccounts((toggle) => !toggle)}
      >
        <TableCell className="flex items-center">
          {Array.from({ length: depth }).map((_, i) => (
            <div key={i} className="w-8" />
          ))}
          <div className="w-8">
            {hasSubAccounts &&
              (showingSubAccounts ? <IoCaretDownSharp /> : <IoCaretForwardSharp />)}
          </div>
          <div>{account.name}</div>
        </TableCell>
      </TableRow>

      {hasSubAccounts && showingSubAccounts && (
        <SubAccountsForAccountSet account={account} depth={depth} />
      )}
    </>
  )
}
