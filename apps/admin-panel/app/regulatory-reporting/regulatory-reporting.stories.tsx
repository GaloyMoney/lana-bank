import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import RegulatoryReportingPage from "./page"

import faker from "@/.storybook/faker"

import { ReportsDocument } from "@/lib/graphql/generated"
import { mockReport } from "@/lib/graphql/generated/mocks"

const createRandomReports = () => {
  const count = faker.number.int({ min: 3, max: 6 })

  return Array.from({ length: count }, () =>
    mockReport({
      lastError: faker.helpers.maybe(() => faker.lorem.sentence(), { probability: 0.3 }),
    }),
  )
}

const baseMocks = [
  {
    request: {
      query: ReportsDocument,
    },
    result: {
      data: {
        reports: createRandomReports(),
      },
    },
  },
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
        query: ReportsDocument,
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
