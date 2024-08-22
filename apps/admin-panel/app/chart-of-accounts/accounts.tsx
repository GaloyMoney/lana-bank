"use client"

import React from "react"
import { gql } from "@apollo/client"
import { IoCaretDownSharp, IoCaretForwardSharp } from "react-icons/io5"

import {
  AccountAmountsByCurrency,
  AccountSetSubAccount,
  useChartOfAccountsAccountSetQuery,
} from "@/lib/graphql/generated"
import { TableCell, TableRow } from "@/components/primitive/table"
import { DateRange } from "@/components/date-range-picker"

gql`
  query ChartOfAccountsAccountSet(
    $accountSetId: UUID!
    $first: Int!
    $after: String
    $from: Timestamp!
    $until: Timestamp
  ) {
    accountSet(accountSetId: $accountSetId, from: $from, until: $until) {
      id
      name
      subAccounts(first: $first, after: $after) {
        edges {
          cursor
          node {
            __typename
            ... on Account {
              __typename
              id
              name
            }
            ... on AccountSet {
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
  dateRange: DateRange
}

const SubAccountsForAccountSet: React.FC<AccountProps> = ({
  account,
  depth = 0,
  dateRange,
}) => {
  const { data, fetchMore } = useChartOfAccountsAccountSetQuery({
    variables: {
      accountSetId: account.id,
      first: 10,
      from: dateRange.from,
    },
    fetchPolicy: "cache-and-network",
  })

  const hasMoreSubAccounts = data?.accountSet?.subAccounts.pageInfo.hasNextPage
  const subAccounts = data?.accountSet?.subAccounts.edges

  return (
    <>
      {subAccounts?.map((subAccount) => (
        <Account
          key={subAccount.node.id}
          account={{
            ...subAccount.node,
            amounts: undefined as unknown as AccountAmountsByCurrency,
          }}
          depth={depth + 1}
          dateRange={dateRange}
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

export const Account: React.FC<AccountProps> = ({ account, depth = 0, dateRange }) => {
  const [showingSubAccounts, setShowingSubAccounts] = React.useState(false)
  const hasSubAccounts = account.__typename === "AccountSet" && account.hasSubAccounts

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
        <SubAccountsForAccountSet account={account} depth={depth} dateRange={dateRange} />
      )}
    </>
  )
}
