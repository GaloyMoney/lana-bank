import React from "react"
import { Meta, StoryFn } from "@storybook/react"

import { Skeleton } from "@/components/primitive/skeleton"

export default {
  title: "Components/Skeleton",
  component: Skeleton,
  argTypes: {
    className: {
      control: "text",
      defaultValue: "",
    },
    width: {
      control: "text",
      defaultValue: "100%",
    },
    height: {
      control: "text",
      defaultValue: "50px",
    },
  },
} as Meta<typeof Skeleton>

const Template: StoryFn<typeof Skeleton> = (args) => <Skeleton {...args} />

export const Default = Template.bind({})
Default.args = {
  className: "w-32 h-32",
}

export const Small = Template.bind({})
Small.args = {
  className: "w-24 h-6",
}

export const Medium = Template.bind({})
Medium.args = {
  className: "w-48 h-12",
}

export const Large = Template.bind({})
Large.args = {
  className: "w-96 h-24",
}

export const FullWidth = Template.bind({})
FullWidth.args = {
  className: "w-full h-12",
}
