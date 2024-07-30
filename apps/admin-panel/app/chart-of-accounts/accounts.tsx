"use client"

import React from "react"
import { gql } from "@apollo/client"
import { IoCaretDownSharp, IoCaretForwardSharp } from "react-icons/io5"

import {
  AccountSetSubAccount,
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
  account: AccountSetSubAccount
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
      {subAccounts?.map((subAccount) => {
        // TODO: change this at the type level when we remove 'Details' from ChartOfAccounts query
        const node =
          subAccount.node.__typename === "AccountWithBalance"
            ? { ...subAccount.node, __typename: "AccountDetails" as const }
            : { ...subAccount.node, __typename: "AccountSetDetails" as const }
        return <Account key={subAccount.node.id} account={node} depth={depth + 1} />
      })}
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
    account.__typename === "AccountSetDetails" && account.hasSubAccounts

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
