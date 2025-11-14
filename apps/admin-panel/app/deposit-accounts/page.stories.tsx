import type { Meta, StoryObj } from "@storybook/nextjs"
import type { MockedResponse } from "@apollo/client/testing"
import { ApolloError } from "@apollo/client"

import DepositAccounts from "./page"

import { DepositAccountsDocument, DepositAccountStatus } from "@/lib/graphql/generated"
import {
  mockDepositAccount,
  mockDepositAccountEdge,
  mockPageInfo,
} from "@/lib/graphql/generated/mocks"

const depositAccountEdges = [
  mockDepositAccountEdge({
    cursor: "cursor-1",
    node: mockDepositAccount({ publicId: "DA-001", status: DepositAccountStatus.Active }),
  }),
  mockDepositAccountEdge({
    cursor: "cursor-2",
    node: mockDepositAccount({ publicId: "DA-002", status: DepositAccountStatus.Frozen }),
  }),
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
        __typename: "DepositAccountConnection",
        edges: depositAccountEdges,
        pageInfo: mockPageInfo({ hasNextPage: false }),
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
        __typename: "DepositAccountConnection",
        edges: [],
        pageInfo: mockPageInfo({ hasNextPage: false, endCursor: null, startCursor: null }),
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
