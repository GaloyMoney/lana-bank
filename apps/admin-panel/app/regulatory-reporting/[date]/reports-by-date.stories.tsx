import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import { ApolloError } from "@apollo/client"
import { toISODateString } from "@lana/web/utils"

import RegulatoryReportByDate from "./page"

import faker from "@/.storybook/faker"

import { ReportsByDateDocument } from "@/lib/graphql/generated"

const createRandomPathInBucket = () => {
  return faker.helpers.arrayElement([
    "reports/2023-10-01/NRP_01/report1.pdf",
    "reports/2023-10-01/NRP_02/report2.pdf",
    "reports/2023-10-02/NRP_03/report3.pdf",
    "reports/2023-10-02/NRP_04/report4.csv",
    "reports/2023-10-02/NRP_05/report5.xml",
  ])
}

const date = toISODateString(faker.date.recent({ days: 30 }))

const RegulatoryReportByDateStory = () => {
  const mocks = [
    {
      request: {
        query: ReportsByDateDocument,
        variables: { date, first: 10 },
      },
      result: {
        data: {
          reportsByDate: {
            edges: Array.from({ length: 10 }, () => ({
              cursor: faker.string.alpha(10),
              node: {
                __typename: "Report",
                id: faker.string.uuid(),
                reportId: faker.string.uuid(),
                date,
                pathInBucket: createRandomPathInBucket(),
              },
            })),
            pageInfo: {
              endCursor: faker.string.alpha(10),
              startCursor: faker.string.alpha(10),
              hasNextPage: faker.datatype.boolean(),
              hasPreviousPage: faker.datatype.boolean(),
            },
          },
        },
      },
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <RegulatoryReportByDate params={Promise.resolve({ date })} />
    </MockedProvider>
  )
}

const meta: Meta = {
  title: "Pages/RegulatoryReporting/ByDate",
  component: RegulatoryReportByDateStory,
  parameters: { layout: "fullscreen", nextjs: { appDirectory: true } },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  parameters: {
    nextjs: {
      navigation: {
        pathname: `/regulatory-reporting/${date}`,
      },
    },
  },
}

export const Error: Story = {
  render: () => {
    const errorMocks = [
      {
        request: {
          query: ReportsByDateDocument,
          variables: { date, first: 10 },
        },
        error: new ApolloError({ errorMessage: faker.lorem.sentence() }),
      },
    ]

    return (
      <MockedProvider mocks={errorMocks} addTypename={false}>
        <RegulatoryReportByDate params={Promise.resolve({ date })} />
      </MockedProvider>
    )
  },
}

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: ReportsByDateDocument,
        variables: { date, first: 10 },
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <RegulatoryReportByDate params={Promise.resolve({ date })} />
    </MockedProvider>
  )
}

export const Loading: Story = {
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: `/regulatory-reporting/${date}`,
      },
    },
  },
}
