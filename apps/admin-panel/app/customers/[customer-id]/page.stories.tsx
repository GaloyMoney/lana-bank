import type { Meta, StoryObj } from "@storybook/nextjs"
import type { MockedResponse } from "@apollo/client/testing"

import CustomerLayout from "./layout"
import CustomerCreditFacilitiesLandingPage from "./page"

import {
  buildParams,
  creditFacilitiesMock,
  customerDetailsMock,
  emptyCreditFacilitiesMock,
} from "./storybook-mocks"

type StoryProps = React.ComponentProps<typeof CustomerCreditFacilitiesLandingPage> & {
  mocks?: MockedResponse[]
}

const meta: Meta<StoryProps> = {
  title: "Pages/Customers/Customer/Landing",
  component: CustomerCreditFacilitiesLandingPage,
  parameters: {
    layout: "fullscreen",
    nextjs: {
      appDirectory: true,
    },
  },
  argTypes: {
    mocks: { control: false },
  },
}

export default meta

type Story = StoryObj<StoryProps>

export const Default: Story = {
  args: {
    params: buildParams(),
    mocks: [customerDetailsMock, creditFacilitiesMock],
  },
  render: ({ params }) => (
    <CustomerLayout params={params}>
      <CustomerCreditFacilitiesLandingPage params={params} />
    </CustomerLayout>
  ),
}

export const Empty: Story = {
  args: {
    params: buildParams(),
    mocks: [customerDetailsMock, emptyCreditFacilitiesMock],
  },
  render: ({ params }) => (
    <CustomerLayout params={params}>
      <CustomerCreditFacilitiesLandingPage params={params} />
    </CustomerLayout>
  ),
}
