import type { Meta, StoryObj } from "@storybook/react"
import { ApolloError } from "@apollo/client"
import { MockedProvider } from "@apollo/client/testing"

import Customers from "./page"

import { AccountStatus, CustomersDocument } from "@/lib/graphql/generated"

const baseMocks = [
  {
    request: {
      query: CustomersDocument,
      variables: {
        first: 10,
        sort: null,
        filter: null,
      },
    },
    result: {
      data: {
        customers: {
          edges: [
            {
              node: {
                id: "1",
                customerId: "CUST001",
                status: AccountStatus.Active,
                level: 1,
                email: "customer1@example.com",
                telegramId: "@customer1",
                applicantId: "APP001",
                balance: {
                  checking: {
                    settled: 1000,
                    pending: 0,
                  },
                },
                subjectCanRecordDeposit: true,
                subjectCanInitiateWithdrawal: true,
                subjectCanCreateCreditFacility: true,
              },
              cursor: "cursor1",
            },
            {
              node: {
                id: "2",
                customerId: "CUST002",
                status: AccountStatus.Inactive,
                level: 1,
                email: "customer2@example.com",
                telegramId: "@customer2",
                applicantId: "APP002",
                balance: {
                  checking: {
                    settled: 500,
                    pending: 100,
                  },
                },
                subjectCanRecordDeposit: false,
                subjectCanInitiateWithdrawal: false,
                subjectCanCreateCreditFacility: false,
              },
              cursor: "cursor2",
            },
          ],
          pageInfo: {
            endCursor: "cursor2",
            startCursor: "cursor1",
            hasNextPage: false,
            hasPreviousPage: false,
          },
        },
      },
    },
  },
]

const meta = {
  title: "Pages/Customers",
  component: Customers,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
} satisfies Meta<typeof Customers>

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
}

export const Error: Story = {
  decorators: [
    (Story) => (
      <MockedProvider
        mocks={[
          {
            request: {
              query: CustomersDocument,
              variables: {
                first: 10,
                sort: null,
                filter: null,
              },
            },
            error: new ApolloError({ errorMessage: "An error occurred" }),
          },
        ]}
        addTypename={false}
      >
        <div className="max-w-7xl m-auto p-4">
          <Story />
        </div>
      </MockedProvider>
    ),
  ],
}

export const Empty: Story = {
  decorators: [
    (Story) => (
      <MockedProvider
        mocks={[
          {
            request: {
              query: CustomersDocument,
              variables: {
                first: 10,
                sort: null,
                filter: null,
              },
            },
            result: {
              data: {
                customers: {
                  edges: [],
                  pageInfo: {
                    endCursor: null,
                    startCursor: null,
                    hasNextPage: false,
                    hasPreviousPage: false,
                  },
                },
              },
            },
          },
        ]}
        addTypename={false}
      >
        <div className="max-w-7xl m-auto p-4">
          <Story />
        </div>
      </MockedProvider>
    ),
  ],
}
