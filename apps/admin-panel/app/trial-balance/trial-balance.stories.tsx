import type { Meta, StoryObj } from "@storybook/react"
import { MockedProvider } from "@apollo/client/testing"

import TrialBalance from "./page"

import {
  GetOnBalanceSheetTrialBalanceDocument,
  GetOffBalanceSheetTrialBalanceDocument,
} from "@/lib/graphql/generated"

import {
  onBalanceSheetTrialBalanceMockData,
  offBalanceSheetTrialBalanceMockData,
} from "@/.storybook/mocks"

const createMocks = () => [
  {
    request: {
      query: GetOnBalanceSheetTrialBalanceDocument,
    },
    variableMatcher: () => true,
    result: onBalanceSheetTrialBalanceMockData,
  },
  {
    request: {
      query: GetOffBalanceSheetTrialBalanceDocument,
    },
    variableMatcher: () => true,
    result: offBalanceSheetTrialBalanceMockData,
  },
]

const TrialBalanceStory = () => {
  const mocks = createMocks()

  return (
    <MockedProvider
      defaultOptions={{ watchQuery: { fetchPolicy: "no-cache" } }}
      mocks={mocks}
      addTypename={false}
    >
      <TrialBalance />
    </MockedProvider>
  )
}

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: GetOnBalanceSheetTrialBalanceDocument,
      },
      variableMatcher: () => true,
      delay: Infinity,
    },
    {
      request: {
        query: GetOffBalanceSheetTrialBalanceDocument,
      },
      variableMatcher: () => true,
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider
      defaultOptions={{ watchQuery: { fetchPolicy: "no-cache" } }}
      mocks={mocks}
      addTypename={false}
    >
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
