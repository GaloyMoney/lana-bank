import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import Committees from "./page"

import faker from "@/.storybook/faker"

import { CommitteesDocument } from "@/lib/graphql/generated"
import { mockCommittee, mockUser } from "@/lib/graphql/generated/mocks"

const createRandomCommittees = () => {
  const count = faker.number.int({ min: 5, max: 10 })

  return Array.from({ length: count }, () => ({
    node: mockCommittee({
      name: faker.company.name(),
      createdAt: faker.date.recent({ days: 30 }).toISOString(),
      currentMembers: Array.from(
        { length: faker.number.int({ min: 2, max: 5 }) },
        (_, i) => mockUser({ id: String(i) }),
      ),
    }),
  }))
}

const baseMocks = [
  {
    request: {
      query: CommitteesDocument,
      variables: {
        first: 10,
      },
    },
    result: {
      data: {
        committees: {
          edges: createRandomCommittees(),
          pageInfo: {
            hasNextPage: false,
            hasPreviousPage: false,
          },
        },
      },
    },
  },
]

const meta = {
  title: "Pages/Committees",
  component: Committees,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
} satisfies Meta<typeof Committees>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/committees",
      },
    },
  },
  decorators: [
    (Story) => (
      <MockedProvider mocks={baseMocks} addTypename={false}>
        <Story />
      </MockedProvider>
    ),
  ],
}

const LoadingStory = () => {
  const mocks = [
    {
      request: {
        query: CommitteesDocument,
        variables: {
          first: 10,
        },
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <Committees />
    </MockedProvider>
  )
}

export const Loading: Story = {
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/committees",
      },
    },
  },
}
