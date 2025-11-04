import type { Meta, StoryObj } from "@storybook/nextjs"

import type { MockedResponse } from "@apollo/client/testing"
import { ApolloError } from "@apollo/client"

import DepositAccounts from "./page"

import {
  DepositAccountStatus,
  DepositAccountsDocument,
} from "@/lib/graphql/generated"

const connectionPageInfo = {
  __typename: "PageInfo" as const,
  endCursor: "cursor-2",
  startCursor: "cursor-1",
  hasNextPage: false,
  hasPreviousPage: false,
}

const depositAccountEdges = [
  {
    __typename: "DepositAccountEdge" as const,
    cursor: "cursor-1",
    node: {
      __typename: "DepositAccount" as const,
      id: "deposit-account-001",
      publicId: "DA-001",
      createdAt: "2024-01-01T12:00:00.000Z",
      status: DepositAccountStatus.Active,
      balance: {
        __typename: "DepositAccountBalance" as const,
        settled: 1_250_000,
        pending: 50_000,
      },
      customer: {
        __typename: "Customer" as const,
        customerId: "customer-001",
        email: "primary@example.com",
        publicId: "CUS-001",
      },
    },
  },
  {
    __typename: "DepositAccountEdge" as const,
    cursor: "cursor-2",
    node: {
      __typename: "DepositAccount" as const,
      id: "deposit-account-002",
      publicId: "DA-002",
      createdAt: "2024-01-05T12:30:00.000Z",
      status: DepositAccountStatus.Frozen,
      balance: {
        __typename: "DepositAccountBalance" as const,
        settled: 750_000,
        pending: 120_000,
      },
      customer: {
        __typename: "Customer" as const,
        customerId: "customer-002",
        email: "secondary@example.com",
        publicId: "CUS-002",
      },
    },
  },
]

const baseMock: MockedResponse = {
  request: {
    query: DepositAccountsDocument,
    variables: {
      first: 10,
    },
  },
  result: {
    data: {
      depositAccounts: {
        __typename: "DepositAccountConnection" as const,
        edges: depositAccountEdges,
        pageInfo: connectionPageInfo,
      },
    },
  },
}

const emptyMock: MockedResponse = {
  request: {
    query: DepositAccountsDocument,
    variables: {
      first: 10,
    },
  },
  result: {
    data: {
      depositAccounts: {
        __typename: "DepositAccountConnection" as const,
        edges: [],
        pageInfo: {
          ...connectionPageInfo,
          endCursor: null,
          startCursor: null,
        },
      },
    },
  },
}

const errorMock: MockedResponse = {
  request: {
    query: DepositAccountsDocument,
    variables: {
      first: 10,
    },
  },
  error: new ApolloError({ errorMessage: "Failed to fetch deposit accounts" }),
}

const loadingMock: MockedResponse = {
  request: {
    query: DepositAccountsDocument,
    variables: {
      first: 10,
    },
  },
  delay: Infinity,
}

type StoryProps = React.ComponentProps<typeof DepositAccounts> & {
  mocks?: MockedResponse[]
}

const meta: Meta<StoryProps> = {
  title: "Pages/DepositAccounts/List",
  component: DepositAccounts,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
  argTypes: {
    mocks: { control: false },
  },
}

export default meta

type Story = StoryObj<StoryProps>

export const Default: Story = {
  args: {
    mocks: [baseMock],
  },
  render: () => <DepositAccounts />,
}

export const Empty: Story = {
  args: {
    mocks: [emptyMock],
  },
  render: () => <DepositAccounts />,
}

export const Error: Story = {
  args: {
    mocks: [errorMock],
  },
  render: () => <DepositAccounts />,
}

export const Loading: Story = {
  args: {
    mocks: [loadingMock],
  },
  render: () => <DepositAccounts />,
}
