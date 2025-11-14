import type { Meta, StoryObj } from "@storybook/nextjs"
import type { MockedResponse } from "@apollo/client/testing"

import CustomerLayout from "../layout"
import {
  buildParams,
  customerDetailsMock,
  customerDocumentsMock,
  emptyCustomerDocumentsMock,
} from "../storybook-mocks"

import CustomerDocumentsPage from "./page"

type StoryProps = React.ComponentProps<typeof CustomerDocumentsPage> & {
  mocks?: MockedResponse[]
}

const meta: Meta<StoryProps> = {
  title: "Pages/Customers/Customer/Documents",
  component: CustomerDocumentsPage,
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

const renderWithLayout = ({ params }: StoryProps) => (
  <CustomerLayout params={params}>
    <CustomerDocumentsPage params={params} />
  </CustomerLayout>
)

export const Default: Story = {
  args: {
    params: buildParams(),
    mocks: [customerDetailsMock, customerDocumentsMock],
  },
  render: renderWithLayout,
}

export const Empty: Story = {
  args: {
    params: buildParams(),
    mocks: [customerDetailsMock, emptyCustomerDocumentsMock],
  },
  render: renderWithLayout,
}
