import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import CreditFacilityLayout from "./layout"

import {
  ApprovalProcessStatus,
  CreditFacilityStatus,
  GetCreditFacilityLayoutDetailsDocument,
} from "@/lib/graphql/generated"
import { mockApprovalProcess, mockCreditFacility } from "@/lib/graphql/generated/mocks"

const meta: Meta<typeof CreditFacilityLayout> = {
  title: "Pages/CreditFacilities/CreditFacility/Layout",
  component: CreditFacilityLayout,
  parameters: { layout: "fullscreen" },
}

export default meta

export const Default: StoryObj<typeof CreditFacilityLayout> = {
  render: () => {
    const creditFacilityId = "test-id"
    const mocks = [
      {
        request: {
          query: GetCreditFacilityLayoutDetailsDocument,
          variables: { id: creditFacilityId },
        },
        result: {
          data: {
            creditFacility: mockCreditFacility({
              status: CreditFacilityStatus.PendingApproval,
              approvalProcess: mockApprovalProcess({
                status: ApprovalProcessStatus.InProgress,
              }),
            }),
          },
        },
      },
    ]

    return (
      <MockedProvider mocks={mocks} addTypename={false}>
        <CreditFacilityLayout
          params={Promise.resolve({ "credit-facility-id": creditFacilityId })}
        >
          <div className="border flex justify-center items-center p-12">TAB CONTENT</div>
        </CreditFacilityLayout>
      </MockedProvider>
    )
  },
}
