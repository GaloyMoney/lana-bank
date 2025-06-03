import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import AuditLogs from "./page"

import { AuditLogsDocument } from "@/lib/graphql/generated"
import { mockPageInfo } from "@/lib/graphql/generated/mocks"

const singleEntry = {
  cursor: "eyJpZCI6NDU5MDJ9",
  node: {
    id: "audit::AuditEntry:45902",
    subject: {
      userId: "7613108b-8d5e-4f01-bfb8-7297324650e2",
      __typename: "User",
      email: "admin@galoy.io",
      roles: ["SUPERUSER"],
    },
    object: "app/audit",
    action: "app:audit:list",
    authorized: true,
    recordedAt: "2024-11-28T10:14:26.758432Z",
    __typename: "AuditEntry",
  },
  __typename: "AuditEntryEdge",
}

const mockAuditEntries = Array.from({ length: 10 }, () => singleEntry)

const baseMocks = [
  {
    request: {
      query: AuditLogsDocument,
      variables: {
        first: 10,
      },
    },
    result: {
      data: {
        audit: {
          edges: mockAuditEntries,
          pageInfo: mockPageInfo({
            hasNextPage: true,
            endCursor: "eyJpZCI6NDU4ODd9",
            __typename: "PageInfo",
          }),
          __typename: "AuditEntryConnection",
        },
      },
    },
  },
]

const meta = {
  title: "Pages/AuditLogs",
  component: AuditLogs,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
} satisfies Meta<typeof AuditLogs>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {
  decorators: [
    (Story) => (
      <MockedProvider mocks={baseMocks} addTypename={true}>
        <Story />
      </MockedProvider>
    ),
  ],
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/audit",
      },
    },
  },
}

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: AuditLogsDocument,
        variables: {
          first: 10,
        },
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <AuditLogs />
    </MockedProvider>
  )
}

export const Loading: Story = {
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/audit",
      },
    },
  },
}
