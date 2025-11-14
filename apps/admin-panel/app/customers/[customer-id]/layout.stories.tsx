import type { Meta, StoryObj } from "@storybook/nextjs"
import type { MockedResponse } from "@apollo/client/testing"

import CustomerLayout from "./layout"

import {
  buildParams,
  customerDetailsLoadingMock,
  customerDetailsMock,
} from "./storybook-mocks"

type StoryProps = React.ComponentProps<typeof CustomerLayout> & {
  mocks?: MockedResponse[]
}

const meta: Meta<StoryProps> = {
  title: "Pages/Customers/Customer/Layout",
  component: CustomerLayout,
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

const tabPlaceholder = (
  <div className="border flex justify-center items-center p-12">TAB CONTENT</div>
)

export const Default: Story = {
  args: {
    params: buildParams(),
    children: tabPlaceholder,
    mocks: [customerDetailsMock],
  },
  render: ({ params, children }) => (
    <CustomerLayout params={params}>{children}</CustomerLayout>
  ),
}

export const Loading: Story = {
  args: {
    params: buildParams(),
    children: tabPlaceholder,
    mocks: [customerDetailsLoadingMock],
  },
  render: ({ params, children }) => (
    <CustomerLayout params={params}>{children}</CustomerLayout>
  ),
}
