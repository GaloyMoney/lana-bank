import type { Meta, StoryObj } from "@storybook/nextjs"
import type { MockedResponse } from "@apollo/client/testing"
import { ApolloError } from "@apollo/client"

import DepositAccountPage from "./page"

import { GetDepositAccountDetailsDocument } from "@/lib/graphql/generated"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import {
  mockDepositAccount,
  mockDepositAccountHistoryEntryEdge,
  mockPageInfo,
} from "@/lib/graphql/generated/mocks"

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

const depositAccount = mockDepositAccount({ publicId: DEPOSIT_ACCOUNT_PUBLIC_ID })

const historyEdges = [
  mockDepositAccountHistoryEntryEdge(),
  mockDepositAccountHistoryEntryEdge(),
]

const baseHistory = {
  __typename: "DepositAccountHistoryEntryConnection",
  edges: historyEdges,
  nodes: historyEdges.map((edge) => edge.node),
  pageInfo: mockPageInfo({ hasNextPage: false, endCursor: null, startCursor: null }),
}

const emptyHistory = {
  __typename: "DepositAccountHistoryEntryConnection",
  edges: [],
  nodes: [],
  pageInfo: mockPageInfo({ hasNextPage: false, endCursor: null, startCursor: null }),
}

const buildHistoryMocks = (history: typeof baseHistory): MockedResponse[] => {
  const data = {
    depositAccountByPublicId: {
      ...depositAccount,
      history,
    },
  }

  const requestWithCursor = (after: string | null) => ({
    query: GetDepositAccountDetailsDocument,
    variables: {
      ...QUERY_VARIABLES,
      after,
    },
  })

  const mocks: MockedResponse[] = [
    {
      request: requestWithCursor(null),
      result: { data },
      newData: () => ({ data }),
    },
  ]

  const endCursor = history.pageInfo?.endCursor
  if (endCursor) {
    mocks.push({
      request: requestWithCursor(endCursor),
      result: { data },
      newData: () => ({ data }),
    })
  }
  return mocks
}

const baseMocks = buildHistoryMocks(baseHistory)
const emptyHistoryMocks = buildHistoryMocks(emptyHistory)

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
