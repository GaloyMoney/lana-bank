import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import WithdrawalPage from "./page"

import faker from "@/.storybook/faker"
import { GetWithdrawalDetailsDocument, WithdrawalStatus } from "@/lib/graphql/generated"
import { mockApprovalProcess, mockWithdrawal } from "@/lib/graphql/generated/mocks"
import { UsdCents } from "@/types"

interface WithdrawalStoryArgs {
  amount: number
  status: WithdrawalStatus
  reference: string
  denialReason: string | null
}

const DEFAULT_ARGS: WithdrawalStoryArgs = {
  amount: faker.number.int({ min: 100, max: 10000 }),
  status: WithdrawalStatus.PendingApproval,
  reference: faker.string.alphanumeric(10),
  denialReason: null,
}

const createMocks = (args: WithdrawalStoryArgs, withdrawalId: string) => [
  {
    request: {
      query: GetWithdrawalDetailsDocument,
      variables: { id: withdrawalId },
    },
    result: {
      data: {
        withdrawal: mockWithdrawal({
          amount: args.amount as UsdCents,
          status: args.status,
          reference: args.reference,
          approvalProcess: mockApprovalProcess({
            deniedReason: args.denialReason,
          }),
        }),
      },
    },
  },
]

const WithdrawalStory = (args: WithdrawalStoryArgs) => {
  const withdrawalId = faker.string.uuid()
  const mocks = createMocks(args, withdrawalId)
  return (
    <MockedProvider mocks={mocks} addTypename={false} key={JSON.stringify(args)}>
      <WithdrawalPage params={Promise.resolve({ "withdrawal-id": withdrawalId })} />
    </MockedProvider>
  )
}

const LoadingStory = () => {
  const withdrawalId = faker.string.uuid()
  const mocks = [
    {
      request: {
        query: GetWithdrawalDetailsDocument,
        variables: { id: withdrawalId },
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <WithdrawalPage params={Promise.resolve({ "withdrawal-id": withdrawalId })} />
    </MockedProvider>
  )
}

const meta = {
  title: "Pages/Withdrawals/Withdrawal/Details",
  component: WithdrawalStory,
  parameters: { layout: "fullscreen", nextjs: { appDirectory: true } },
  argTypes: {
    amount: {
      control: "number",
      description: "Withdrawal amount",
    },
    status: {
      control: "select",
      options: Object.values(WithdrawalStatus),
      description: "Withdrawal status",
    },
    reference: {
      control: "text",
      description: "Withdrawal reference",
    },
    denialReason: {
      control: "text",
      description: "Denial reason",
    },
  },
} satisfies Meta<typeof WithdrawalStory>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: DEFAULT_ARGS,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/withdrawals/[withdrawal-id]",
      },
    },
  },
}

export const Loading: Story = {
  args: DEFAULT_ARGS,
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/withdrawals/[withdrawal-id]",
      },
    },
  },
}
