import React from "react"
import { Meta, StoryFn } from "@storybook/react"

import { Select, SelectProps } from "@/components/primitive/select"

export default {
  title: "Components/Select",
  component: Select,
  argTypes: {
    disabled: {
      control: "boolean",
      description: "Controls whether the select is disabled",
    },
  },
} as Meta<typeof Select>

const Template: StoryFn<SelectProps> = (args) => <Select {...args} />

export const Default = Template.bind({})
Default.args = {
  children: (
    <>
      <option value="" disabled selected>
        Select an option...
      </option>
      <option value="option1">Option 1</option>
      <option value="option2">Option 2</option>
      <option value="option3">Option 3</option>
    </>
  ),
}
