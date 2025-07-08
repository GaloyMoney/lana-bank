import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"
import { toISODateString } from "@lana/web/utils"

import RegulatoryReportingPage from "./page"

import faker from "@/.storybook/faker"

import { ReportListAvailableDatesDocument } from "@/lib/graphql/generated"

const createRandomReportDates = () => {
  const count = faker.number.int({ min: 3, max: 6 })

  return Array.from({ length: count }, () => {
    const date = faker.date.recent({ days: 30 })
    return toISODateString(date)
  })
}

const baseMocks = [
  {
    request: {
      query: ReportListAvailableDatesDocument,
    },
    result: {
      data: {
        reportListAvailableDates: createRandomReportDates(),
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
        query: ReportListAvailableDatesDocument,
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
