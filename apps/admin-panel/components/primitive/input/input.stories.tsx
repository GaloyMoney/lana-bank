import React from "react"
import { Meta, StoryFn } from "@storybook/react"

import { Input, InputProps } from "@/components/primitive/input"

export default {
  title: "Components/Input",
  component: Input,
  argTypes: {
    type: {
      control: "text",
      description: "Defines the type of the input",
    },
    placeholder: {
      control: "text",
      description: "Placeholder text",
    },
    disabled: {
      control: "boolean",
      description: "Controls whether the input is disabled",
    },
    value: {
      control: "text",
      description: "Value of the input",
    },
    onChange: {
      action: "changed",
      description: "Event handler for when the value changes",
    },
  },
} as Meta<typeof Input>

const Template: StoryFn<InputProps> = (args) => <Input {...args} />

export const Default = Template.bind({})
Default.args = {
  type: "text",
  placeholder: "Enter text here...",
}
