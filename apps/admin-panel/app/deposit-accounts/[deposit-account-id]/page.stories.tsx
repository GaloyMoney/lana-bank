import type { Meta, StoryObj } from "@storybook/nextjs"

import type { MockedResponse } from "@apollo/client/testing"
import { ApolloError } from "@apollo/client"

import DepositAccountPage from "./page"

import {
  DepositAccountStatus,
  DepositStatus,
  DisbursalStatus,
  GetDepositAccountDetailsDocument,
  WithdrawalStatus,
} from "@/lib/graphql/generated"

const DEPOSIT_ACCOUNT_PUBLIC_ID = "DA-001"

const buildParams = () =>
  Promise.resolve({
    "deposit-account-id": DEPOSIT_ACCOUNT_PUBLIC_ID,
  })

const QUERY_VARIABLES = {
  publicId: DEPOSIT_ACCOUNT_PUBLIC_ID,
  first: 20,
  after: null,
}

const depositAccountHistoryEdges = [
  {
    __typename: "DepositAccountHistoryEntryEdge" as const,
    cursor: "cursor-1",
    node: {
      __typename: "DepositEntry" as const,
      recordedAt: "2024-02-01T09:30:00.000Z",
      deposit: {
        __typename: "Deposit" as const,
        id: "deposit-001",
        depositId: "deposit-001",
        publicId: "DEP-001",
        accountId: "account-001",
        amount: 150_000,
        createdAt: "2024-02-01T09:00:00.000Z",
        reference: "Incoming wire",
        status: DepositStatus.Confirmed,
        description: "Seed capital deposit",
      },
    },
  },
  {
    __typename: "DepositAccountHistoryEntryEdge" as const,
    cursor: "cursor-2",
    node: {
      __typename: "WithdrawalEntry" as const,
      recordedAt: "2024-02-03T15:45:00.000Z",
      withdrawal: {
        __typename: "Withdrawal" as const,
        id: "withdrawal-001",
        withdrawalId: "withdrawal-001",
        publicId: "WIT-001",
        accountId: "account-001",
        amount: 50_000,
        createdAt: "2024-02-03T15:10:00.000Z",
        reference: "Client cash out",
        status: WithdrawalStatus.Confirmed,
        description: "Withdrawal to treasury wallet",
      },
    },
  },
  {
    __typename: "DepositAccountHistoryEntryEdge" as const,
    cursor: "cursor-2a",
    node: {
      __typename: "CancelledWithdrawalEntry" as const,
      recordedAt: "2024-02-04T08:00:00.000Z",
      withdrawal: {
        __typename: "Withdrawal" as const,
        id: "withdrawal-002",
        withdrawalId: "withdrawal-002",
        publicId: "WIT-002",
        accountId: "account-001",
        amount: 35_000,
        createdAt: "2024-02-04T07:45:00.000Z",
        reference: "Cancelled cash out",
        status: WithdrawalStatus.Cancelled,
        description: "Withdrawal cancelled by operator",
      },
    },
  },
  {
    __typename: "DepositAccountHistoryEntryEdge" as const,
    cursor: "cursor-3",
    node: {
      __typename: "DisbursalEntry" as const,
      recordedAt: "2024-02-05T11:00:00.000Z",
      disbursal: {
        __typename: "CreditFacilityDisbursal" as const,
        id: "disbursal-001",
        disbursalId: "disbursal-001",
        publicId: "DIS-001",
        amount: 200_000,
        createdAt: "2024-02-05T10:30:00.000Z",
        status: DisbursalStatus.Confirmed,
        description: "Disbursal for credit facility CF-001",
      },
    },
  },
]

const historyPageInfo = {
  __typename: "PageInfo" as const,
  endCursor:
    depositAccountHistoryEdges[depositAccountHistoryEdges.length - 1]?.cursor ?? null,
  startCursor: depositAccountHistoryEdges[0]?.cursor ?? null,
  hasNextPage: false,
  hasPreviousPage: false,
}

const depositAccountResponse = {
  __typename: "DepositAccount" as const,
  id: "deposit-account-001",
  depositAccountId: "deposit-account-001",
  publicId: DEPOSIT_ACCOUNT_PUBLIC_ID,
  createdAt: "2023-12-15T12:00:00.000Z",
  status: DepositAccountStatus.Active,
  balance: {
    __typename: "DepositAccountBalance" as const,
    settled: 500_000,
    pending: 75_000,
  },
  ledgerAccounts: {
    __typename: "DepositAccountLedgerAccounts" as const,
    depositAccountId: "ledger-account-001",
    frozenDepositAccountId: "ledger-account-002",
  },
  customer: {
    __typename: "Customer" as const,
    id: "customer-001",
    customerId: "customer-001",
    publicId: "CUS-001",
    applicantId: "APP-001",
    email: "customer@example.com",
  },
  history: {
    __typename: "DepositAccountHistoryEntryConnection" as const,
    edges: [] as typeof depositAccountHistoryEdges,
    pageInfo: {
      ...historyPageInfo,
      endCursor: "",
      startCursor: "",
    },
  },
}

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
  __typename: "DepositAccountHistoryEntryConnection" as const,
  edges: depositAccountHistoryEdges,
  pageInfo: historyPageInfo,
})

const emptyHistoryMocks = buildSuccessMocks({
  __typename: "DepositAccountHistoryEntryConnection" as const,
  edges: [],
  pageInfo: {
    ...historyPageInfo,
    endCursor: "",
    startCursor: "",
  },
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
