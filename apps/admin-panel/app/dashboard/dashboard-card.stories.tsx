import { StoryObj } from "@storybook/nextjs"

import DashboardCard from "./dashboard-card"

const meta = {
  title: "Components/DashboardCard",
  component: DashboardCard,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
}

export default meta
type Story = StoryObj<typeof DashboardCard>

export const Basic: Story = {
  args: {
    title: "Analytics Overview",
    description: "View your dashboard analytics and metrics",
    to: "/analytics",
  },
}

export const WithHeadings: Story = {
  args: {
    h1: "2,547",
    h2: "+12.5%",
    h2PopupDescription: "12.5% increase from last month",
    title: "Total Users",
    description: "Active users in your platform",
    to: "/users",
  },
}

export const WithCustomContent: Story = {
  args: {
    title: "Recent Activity",
    description: "Your latest platform activities",
    content: (
      <div className="space-y-2">
        <div className="flex justify-between items-center">
          <span>New sign-ups</span>
          <span className="font-medium">24</span>
        </div>
        <div className="flex justify-between items-center">
          <span>Active sessions</span>
          <span className="font-medium">156</span>
        </div>
        <div className="flex justify-between items-center">
          <span>Completed tasks</span>
          <span className="font-medium">89</span>
        </div>
      </div>
    ),
    to: "/activity",
  },
}

export const RightAlignedButton: Story = {
  args: {
    title: "Performance Metrics",
    description: "Monitor your application performance",
    buttonToRight: true,
    to: "/performance",
  },
}

export const CustomButtonText: Story = {
  args: {
    title: "Revenue Overview",
    description: "Track your monthly revenue and transactions",
    buttonText: "View Revenue Details",
    to: "/revenue",
  },
}

export const NoButton: Story = {
  args: {
    title: "Quick Stats",
    description: "Key metrics at a glance",
    content: (
      <div className="grid grid-cols-2 gap-4">
        <div className="text-center">
          <div className="text-2xl font-bold">98%</div>
          <div className="text-sm text-muted-foreground">Uptime</div>
        </div>
        <div className="text-center">
          <div className="text-2xl font-bold">1.2s</div>
          <div className="text-sm text-muted-foreground">Load Time</div>
        </div>
      </div>
    ),
  },
}

export const ComplexHeadings: Story = {
  args: {
    h1: (
      <div className="flex items-center gap-2">
        <span>$52,000</span>
        <span className="text-sm text-green-500">↑</span>
      </div>
    ),
    h2: "vs last month",
    h2PopupDescription: "Comparison with previous month's revenue",
    title: "Monthly Revenue",
    description: "Total revenue generated this month",
    to: "/revenue/details",
    buttonText: "View Revenue Breakdown",
  },
}
