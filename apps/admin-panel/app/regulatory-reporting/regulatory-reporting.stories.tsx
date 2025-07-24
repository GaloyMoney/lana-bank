import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import RegulatoryReportingPage from "./page"

import { ReportRunsDocument } from "@/lib/graphql/generated"
import { mockReportRunConnection } from "@/lib/graphql/generated/mocks"

const one = {
  request: { query: ReportRunsDocument, variables: { first: 10 } },
  result: { data: { reportRuns: mockReportRunConnection() } },
}

const baseMocks = [
  // polling consumes one mock every request
  ...Array.from({ length: 100 }, () => one),
]

const meta = {
  title: "Pages/RegulatoryReporting",
  component: RegulatoryReportingPage,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
} satisfies Meta<typeof RegulatoryReportingPage>

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
        pathname: "/regulatory-reporting",
      },
    },
  },
}

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: ReportRunsDocument,
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <RegulatoryReportingPage />
    </MockedProvider>
  )
}

export const Loading: Story = {
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/regulatory-reporting",
      },
    },
  },
}
