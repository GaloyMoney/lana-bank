import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import DisbursalPage from "./page"

import faker from "@/.storybook/faker"

import { GetDisbursalDetailsDocument, DisbursalStatus } from "@/lib/graphql/generated"
import {
  mockApprovalProcess,
  mockApprovalProcessVoter,
  mockCreditFacilityDisbursal,
} from "@/lib/graphql/generated/mocks"
import { UsdCents } from "@/types"

interface DisbursalStoryArgs {
  amount: number
  status: DisbursalStatus
  numberOfVoters: number
  showApprovalProcess: boolean
}

const DEFAULT_ARGS: DisbursalStoryArgs = {
  amount: faker.number.int({ min: 10000, max: 1000000 }),
  status: DisbursalStatus.New,
  numberOfVoters: faker.number.int({ min: 2, max: 5 }),
  showApprovalProcess: true,
}

const createMocks = (args: DisbursalStoryArgs, disbursalId: string) => [
  {
    request: {
      query: GetDisbursalDetailsDocument,
      variables: { id: disbursalId },
    },
    result: {
      data: {
        disbursal: mockCreditFacilityDisbursal({
          status: args.status,
          amount: args.amount as UsdCents,
          approvalProcess: mockApprovalProcess({
            voters: Array.from({ length: args.numberOfVoters }, () =>
              mockApprovalProcessVoter(),
            ),
          }),
        }),
      },
    },
  },
]

const DisbursalStory = (args: DisbursalStoryArgs) => {
  const disbursalId = faker.string.uuid()
  const mocks = createMocks(args, disbursalId)

  return (
    <MockedProvider mocks={mocks} addTypename={false} key={JSON.stringify(args)}>
      <DisbursalPage params={Promise.resolve({ "disbursal-id": disbursalId })} />
    </MockedProvider>
  )
}

const LoadingStory = () => {
  const disbursalId = faker.string.uuid()
  const mocks = [
    {
      request: {
        query: GetDisbursalDetailsDocument,
        variables: { id: disbursalId },
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <DisbursalPage params={Promise.resolve({ "disbursal-id": disbursalId })} />
    </MockedProvider>
  )
}

const meta: Meta<typeof DisbursalStory> = {
  title: "Pages/Disbursals/Disbursal/Details",
  component: DisbursalStory,
  parameters: { layout: "fullscreen", nextjs: { appDirectory: true } },
  argTypes: {
    amount: {
      control: { type: "number", min: 1000, max: 1000000 },
      description: "Disbursal amount in cents",
    },
    status: {
      control: "select",
      options: Object.values(DisbursalStatus),
      description: "Approval process status",
    },
    numberOfVoters: {
      control: { type: "number", min: 0, max: 10 },
      description: "Number of voters to display",
    },
    showApprovalProcess: {
      control: "boolean",
      description: "Show approval process section",
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
        pathname: "/disbursals/[disbursals-id]",
      },
    },
  },
}

export const Loading: Story = {
  args: DEFAULT_ARGS,
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/disbursals/[disbursals-id]",
      },
    },
  },
}
