import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import { ApolloError } from "@apollo/client"

import LedgerTransactionPage from "./page"

import faker from "@/.storybook/faker"

import { LedgerTransactionDocument } from "@/lib/graphql/generated"
import { mockLedgerTransaction } from "@/lib/graphql/generated/mocks"

const ledgerTransactionId = faker.string.uuid()

const LedgerTransactionStory = () => {
  const mocks = [
    {
      request: {
        query: LedgerTransactionDocument,
        variables: { id: ledgerTransactionId },
      },
      result: {
        data: {
          ledgerTransaction: mockLedgerTransaction(),
        },
      },
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <LedgerTransactionPage
        params={Promise.resolve({ "ledger-transaction-id": ledgerTransactionId })}
      />
    </MockedProvider>
  )
}

const meta: Meta = {
  title: "Pages/LedgerTransaction",
  component: LedgerTransactionStory,
  parameters: { layout: "fullscreen", nextjs: { appDirectory: true } },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  parameters: {
    nextjs: {
      navigation: {
        pathname: `/ledger-transaction/${ledgerTransactionId}`,
      },
    },
  },
}

export const Error: Story = {
  render: () => {
    const errorMocks = [
      {
        request: {
          query: LedgerTransactionDocument,
          variables: { id: ledgerTransactionId },
        },
        error: new ApolloError({ errorMessage: faker.lorem.sentence() }),
      },
    ]

    return (
      <MockedProvider mocks={errorMocks} addTypename={false}>
        <LedgerTransactionPage
          params={Promise.resolve({ "ledger-transaction-id": ledgerTransactionId })}
        />
      </MockedProvider>
    )
  },
}

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: LedgerTransactionDocument,
        variables: { id: ledgerTransactionId },
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <LedgerTransactionPage
        params={Promise.resolve({ "ledger-transaction-id": ledgerTransactionId })}
      />
    </MockedProvider>
  )
}

export const Loading: Story = {
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: `/ledger-transaction/${ledgerTransactionId}`,
      },
    },
  },
}
