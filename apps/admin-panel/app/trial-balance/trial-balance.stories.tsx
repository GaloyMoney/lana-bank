import type { Meta, StoryObj } from "@storybook/react"
import { MockedProvider } from "@apollo/client/testing"

import TrialBalance from "./page"

import { GetTrialBalanceDocument } from "@/lib/graphql/generated"

import { trialBalanceMockData } from "@/.storybook/mocks"

const createMocks = () => [
  {
    request: {
      query: GetTrialBalanceDocument,
    },
    variableMatcher: () => true,
    result: trialBalanceMockData,
  },
]

const TrialBalanceStory = () => {
  const mocks = createMocks()

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <TrialBalance />
    </MockedProvider>
  )
}

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: GetTrialBalanceDocument,
      },
      variableMatcher: () => true,
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <TrialBalance />
    </MockedProvider>
  )
}

const meta = {
  title: "Pages/TrialBalance",
  component: TrialBalanceStory,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
      navigation: {
        pathname: "/trial-balance",
      },
    },
  },
} satisfies Meta<typeof TrialBalance>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Loading: Story = {
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/trial-balance",
      },
    },
  },
}
