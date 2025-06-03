import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import Withdrawals from "./page"

import faker from "@/.storybook/faker"

import { WithdrawalsDocument } from "@/lib/graphql/generated"
import { mockWithdrawal, mockPageInfo } from "@/lib/graphql/generated/mocks"

const createRandomWithdrawals = () => {
  const count = faker.number.int({ min: 5, max: 10 })

  return Array.from({ length: count }, () => ({
    node: mockWithdrawal(),
  }))
}

const baseMocks = [
  {
    request: {
      query: WithdrawalsDocument,
      variables: {
        first: 10,
      },
    },
    result: {
      data: {
        withdrawals: {
          edges: createRandomWithdrawals(),
          pageInfo: mockPageInfo(),
        },
      },
    },
  },
]

const meta = {
  title: "Pages/Withdrawals",
  component: Withdrawals,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
} satisfies Meta<typeof Withdrawals>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  decorators: [
    (Story) => (
      <MockedProvider mocks={baseMocks} addTypename={false}>
        <Story />
      </MockedProvider>
    ),
  ],
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/withdrawals",
      },
    },
  },
}

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: WithdrawalsDocument,
        variables: {
          first: 10,
        },
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <Withdrawals />
    </MockedProvider>
  )
}

export const Loading: Story = {
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/withdrawals",
      },
    },
  },
}
