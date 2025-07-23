import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import { ApolloError } from "@apollo/client"

import ReportRunById from "./page"

import faker from "@/.storybook/faker"
import { mockReportRun } from "@/lib/graphql/generated/mocks"

import { ReportRunByIdDocument } from "@/lib/graphql/generated"

const reportRunId = faker.string.uuid()

const ReportByIdStory = () => {
  const mocks = [
    {
      request: {
        query: ReportRunByIdDocument,
        variables: { reportRunId },
      },
      result: {
        data: {
          reportRun: mockReportRun(),
        },
      },
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <ReportRunById params={Promise.resolve({ "report-run-id": reportRunId })} />
    </MockedProvider>
  )
}

const meta: Meta = {
  title: "Pages/RegulatoryReporting/ById",
  component: ReportByIdStory,
  parameters: { layout: "fullscreen", nextjs: { appDirectory: true } },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  parameters: {
    nextjs: {
      navigation: {
        pathname: `/regulatory-reporting/${reportRunId}`,
      },
    },
  },
}

export const Error: Story = {
  render: () => {
    const errorMocks = [
      {
        request: {
          query: ReportRunByIdDocument,
          variables: { reportRunId },
        },
        error: new ApolloError({ errorMessage: faker.lorem.sentence() }),
      },
    ]

    return (
      <MockedProvider mocks={errorMocks} addTypename={false}>
        <ReportRunById params={Promise.resolve({ "report-run-id": reportRunId })} />
      </MockedProvider>
    )
  },
}

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: ReportRunByIdDocument,
        variables: { reportRunId },
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <ReportRunById params={Promise.resolve({ "report-run-id": reportRunId })} />
    </MockedProvider>
  )
}

export const Loading: Story = {
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: `/regulatory-reporting/${reportRunId}`,
      },
    },
  },
}
