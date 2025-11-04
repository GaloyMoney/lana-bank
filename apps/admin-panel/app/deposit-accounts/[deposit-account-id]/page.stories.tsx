import type { Meta, StoryObj } from "@storybook/nextjs"
import type { MockedResponse } from "@apollo/client/testing"
import { ApolloError } from "@apollo/client"

import DepositAccountPage from "./page"

import {
  DepositAccountStatus,
  DepositStatus,
  GetDepositAccountDetailsDocument,
  WithdrawalStatus,
} from "@/lib/graphql/generated"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import {
  mockDepositAccount,
  mockDepositAccountHistoryEntryEdge,
  mockDeposit,
  mockWithdrawal,
  mockPageInfo,
} from "@/lib/graphql/generated/mocks"
import type { UsdCents } from "types"

const DEPOSIT_ACCOUNT_PUBLIC_ID = "DA-001"

const buildParams = () =>
  Promise.resolve({
    "deposit-account-id": DEPOSIT_ACCOUNT_PUBLIC_ID,
  })

const QUERY_VARIABLES = {
  publicId: DEPOSIT_ACCOUNT_PUBLIC_ID,
  first: DEFAULT_PAGESIZE,
  after: null,
}

const depositAccountHistoryEdges = [
  mockDepositAccountHistoryEntryEdge({
    cursor: "cursor-1",
    node: {
      __typename: "DepositEntry",
      recordedAt: "2024-02-01T09:30:00.000Z",
      deposit: mockDeposit({
        publicId: "DEP-001",
        amount: 150_000 as UsdCents,
        status: DepositStatus.Confirmed,
      }),
    },
  }),
  mockDepositAccountHistoryEntryEdge({
    cursor: "cursor-2",
    node: {
      __typename: "WithdrawalEntry",
      recordedAt: "2024-02-03T15:45:00.000Z",
      withdrawal: mockWithdrawal({
        publicId: "WIT-001",
        amount: 50_000 as UsdCents,
        status: WithdrawalStatus.Confirmed,
      }),
    },
  }),
]

const depositAccountResponse = mockDepositAccount({
  publicId: DEPOSIT_ACCOUNT_PUBLIC_ID,
  status: DepositAccountStatus.Active,
  history: {
    __typename: "DepositAccountHistoryEntryConnection",
    edges: [],
    nodes: [],
    pageInfo: mockPageInfo({ hasNextPage: false, endCursor: "", startCursor: "" }),
  },
})

const buildSuccessMocks = (
  history: typeof depositAccountResponse.history,
): MockedResponse[] => {
  const result = {
    depositAccountByPublicId: {
      ...depositAccountResponse,
      history,
    },
  }

  const mocks: MockedResponse[] = [
    {
      request: {
        query: GetDepositAccountDetailsDocument,
        variables: QUERY_VARIABLES,
      },
      result: {
        data: result,
      },
    },
  ]

  const endCursor = history?.pageInfo?.endCursor ?? null
  if (endCursor) {
    mocks.push({
      request: {
        query: GetDepositAccountDetailsDocument,
        variables: {
          ...QUERY_VARIABLES,
          after: endCursor,
        },
      },
      result: {
        data: result,
      },
    })
  }

  return mocks
}

const baseMocks = buildSuccessMocks({
  __typename: "DepositAccountHistoryEntryConnection",
  edges: depositAccountHistoryEdges,
  nodes: depositAccountHistoryEdges.map((e) => e.node),
  pageInfo: mockPageInfo({ hasNextPage: false }),
})

const emptyHistoryMocks = buildSuccessMocks({
  __typename: "DepositAccountHistoryEntryConnection",
  edges: [],
  nodes: [],
  pageInfo: mockPageInfo({ hasNextPage: false, endCursor: "", startCursor: "" }),
})

const errorMock: MockedResponse = {
  request: {
    query: GetDepositAccountDetailsDocument,
    variables: QUERY_VARIABLES,
  },
  error: new ApolloError({ errorMessage: "Failed to load deposit account" }),
}

const loadingMock: MockedResponse = {
  request: {
    query: GetDepositAccountDetailsDocument,
    variables: QUERY_VARIABLES,
  },
  delay: Infinity,
}

type StoryProps = React.ComponentProps<typeof DepositAccountPage> & {
  mocks?: MockedResponse[]
}

const meta: Meta<StoryProps> = {
  title: "Pages/DepositAccounts/Details",
  component: DepositAccountPage,
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
    params: buildParams(),
    mocks: baseMocks,
  },
  render: ({ params }) => <DepositAccountPage params={params} />,
}

export const EmptyTransactions: Story = {
  args: {
    params: buildParams(),
    mocks: emptyHistoryMocks,
  },
  render: ({ params }) => <DepositAccountPage params={params} />,
}

export const Error: Story = {
  args: {
    params: buildParams(),
    mocks: [errorMock],
  },
  render: ({ params }) => <DepositAccountPage params={params} />,
}

export const Loading: Story = {
  args: {
    params: buildParams(),
    mocks: [loadingMock],
  },
  render: ({ params }) => <DepositAccountPage params={params} />,
}
