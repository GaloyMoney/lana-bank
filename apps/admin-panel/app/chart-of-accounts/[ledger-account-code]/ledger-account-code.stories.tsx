import type { Meta, StoryObj } from "@storybook/react"
import { MockedProvider } from "@apollo/client/testing"

import { ApolloError } from "@apollo/client"

import LedgerAccountPage from "./page"

import faker from "@/.storybook/faker"

import { LedgerAccountByCodeDocument } from "@/lib/graphql/generated"

const LedgerAccountStory = () => {
  const ledgerAccountCode = String(faker.number.int(10))
  const mocks = [
    {
      request: {
        query: LedgerAccountByCodeDocument,
        variables: { code: ledgerAccountCode, first: 10 },
      },
      result: {
        data: {
          ledgerAccountByCode: {
            id: faker.string.uuid(),
            name: faker.company.name(),
            code: ledgerAccountCode,
            history: {
              edges: Array.from({ length: 10 }, () => ({
                cursor: faker.string.alpha(10),
                node: {
                  __typename: "BtcLedgerAccountHistoryEntry",
                  txId: faker.string.uuid(),
                  recordedAt: faker.date.past().toISOString(),
                  btcAmount: {
                    settled: {
                      debit: faker.finance.amount(),
                      credit: faker.finance.amount(),
                    },
                  },
                },
              })),
              pageInfo: {
                endCursor: faker.string.alpha(10),
                startCursor: faker.string.alpha(10),
                hasNextPage: faker.datatype.boolean(),
                hasPreviousPage: faker.datatype.boolean(),
              },
            },
          },
        },
      },
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <LedgerAccountPage params={{ "ledger-account-code": ledgerAccountCode }} />
    </MockedProvider>
  )
}

const meta: Meta = {
  title: "Pages/ChartOfAccounts/LedgerAccountDetails",
  component: LedgerAccountStory,
  parameters: { layout: "fullscreen", nextjs: { appDirectory: true } },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/committees/[ledger-account-code]",
      },
    },
  },
}

export const Error: Story = {
  render: () => {
    const ledgerAccountCode = String(faker.number.int(10))
    const errorMocks = [
      {
        request: {
          query: LedgerAccountByCodeDocument,
          variables: { code: ledgerAccountCode, first: 10 },
        },
        error: new ApolloError({ errorMessage: faker.lorem.sentence() }),
      },
    ]

    return (
      <MockedProvider mocks={errorMocks} addTypename={false}>
        <LedgerAccountPage params={{ "ledger-account-code": ledgerAccountCode }} />
      </MockedProvider>
    )
  },
}

const LoadingStory = () => {
  const ledgerAccountCode = String(faker.number.int(10))
  const mocks = [
    {
      request: {
        query: LedgerAccountByCodeDocument,
        variables: { code: ledgerAccountCode, first: 10 },
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <LedgerAccountPage params={{ "ledger-account-code": ledgerAccountCode }} />
    </MockedProvider>
  )
}

export const Loading: Story = {
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/chart-of-accounts/[ledger-account-code]",
      },
    },
  },
}
