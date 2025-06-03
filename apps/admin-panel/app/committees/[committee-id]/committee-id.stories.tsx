import type { Meta, StoryObj } from "@storybook/nextjs"
import { ApolloError } from "@apollo/client"
import { MockedProvider } from "@apollo/client/testing"

import CommitteePage from "./page"

import faker from "@/.storybook/faker"

import { GetCommitteeDetailsDocument } from "@/lib/graphql/generated"
import { mockCommittee, mockUser } from "@/lib/graphql/generated/mocks"

interface CommitteeStoryArgs {
  numberOfMembers: number
  committeeName: string
  showEmptyMembers: boolean
}

const DEFAULT_ARGS: CommitteeStoryArgs = {
  numberOfMembers: faker.number.int({ min: 3, max: 8 }),
  committeeName: faker.company.name(),
  showEmptyMembers: false,
}

const createMembers = (args: CommitteeStoryArgs) => {
  if (args.showEmptyMembers) return []

  return Array.from({ length: args.numberOfMembers }, () =>
    mockUser({
      userId: faker.string.uuid(),
      email: faker.internet.email(),
    }),
  )
}

const createMocks = (args: CommitteeStoryArgs, committeeId: string) => [
  {
    request: {
      query: GetCommitteeDetailsDocument,
      variables: { id: committeeId },
    },
    result: {
      data: {
        committee: mockCommittee({
          createdAt: faker.date.past().toISOString(),
          name: args.committeeName,
          currentMembers: createMembers(args),
        }),
      },
    },
  },
]

const CommitteeStory = (args: CommitteeStoryArgs) => {
  const committeeId = faker.string.uuid()
  const mocks = createMocks(args, committeeId)

  return (
    <MockedProvider mocks={mocks} addTypename={false} key={JSON.stringify(args)}>
      <CommitteePage params={Promise.resolve({ "committee-id": committeeId })} />
    </MockedProvider>
  )
}

const meta: Meta<typeof CommitteeStory> = {
  title: "Pages/Committees/Committee/Details",
  component: CommitteeStory,
  parameters: { layout: "fullscreen", nextjs: { appDirectory: true } },
  argTypes: {
    numberOfMembers: {
      control: { type: "number", min: 0, max: 20 },
      description: "Number of committee members to display",
    },
    committeeName: {
      control: "text",
      description: "Name of the committee",
    },
    showEmptyMembers: {
      control: "boolean",
      description: "Show empty state for members list",
    },
  },
}

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: DEFAULT_ARGS,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/committees/[committee-id]",
      },
    },
  },
}

export const Empty: Story = {
  args: {
    ...DEFAULT_ARGS,
    showEmptyMembers: true,
  },
}

export const Error: Story = {
  args: DEFAULT_ARGS,
  render: () => {
    const errorMocks = [
      {
        request: {
          query: GetCommitteeDetailsDocument,
          variables: { id: faker.string.uuid() },
        },
        error: new ApolloError({ errorMessage: faker.lorem.sentence() }),
      },
    ]

    return (
      <MockedProvider mocks={errorMocks} addTypename={false}>
        <CommitteePage
          params={Promise.resolve({ "committee-id": faker.string.uuid() })}
        />
      </MockedProvider>
    )
  },
}

const LoadingStory = () => {
  const committeeId = faker.string.uuid()
  const mocks = [
    {
      request: {
        query: GetCommitteeDetailsDocument,
        variables: { id: committeeId },
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <CommitteePage params={Promise.resolve({ "committee-id": committeeId })} />
    </MockedProvider>
  )
}

export const Loading: Story = {
  args: DEFAULT_ARGS,
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/committees/[committee-id]",
      },
    },
  },
}
