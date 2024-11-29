import React from "react"
import type { Preview } from "@storybook/react"
import { MockedProvider } from "@apollo/client/testing"
import "../app/globals.css"
import { AppSidebar } from "../components/app-sidebar"
import { SidebarInset, SidebarProvider } from "../ui/sidebar"
import {
  AvatarDocument,
  GetRealtimePriceUpdatesDocument,
  Role,
} from "../lib/graphql/generated"
import { mockRealtimePrice } from "../lib/graphql/generated/mocks"
import { AppLayout } from "../app/app-layout"

const defaultMocks = [
  {
    request: { query: AvatarDocument },
    result: {
      data: {
        me: {
          user: {
            userId: "usr_123",
            email: "test@example.com",
            roles: [Role.Admin],
          },
        },
      },
    },
  },
  {
    request: { query: GetRealtimePriceUpdatesDocument },
    result: {
      data: {
        realtimePrice: mockRealtimePrice({
          usdCentsPerBtc: 100000,
        }),
      },
    },
  },
]

const StorybookWrapper = ({ children, mocks = [] }) => (
  <div className="antialiased select-none bg-background">
    <MockedProvider mocks={[...defaultMocks, ...mocks]} addTypename={false}>
      <SidebarProvider>
        <AppSidebar />
        <SidebarInset className="min-h-screen md:peer-data-[variant=inset]:shadow-none border">
          <AppLayout>{children}</AppLayout>
        </SidebarInset>
      </SidebarProvider>
    </MockedProvider>
  </div>
)

const preview: Preview = {
  parameters: {
    nextjs: {
      appDirectory: true,
    },
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
  },
  decorators: [
    (Story, context) => {
      if (context.title.startsWith("Pages/")) {
        const storyMocks = context?.args?.mocks || []
        return (
          <StorybookWrapper mocks={storyMocks}>
            <Story />
          </StorybookWrapper>
        )
      }
      return (
        <div className="max-w-7xl m-auto p-4">
          <Story />
        </div>
      )
    },
  ],
}

export default preview
