import React from "react"
import { Meta, StoryFn } from "@storybook/react"

import { Label } from "@/components/primitive/label"

import { RadioGroup, RadioGroupItem } from "@/components/primitive/radio-group"

export default {
  title: "Components/RadioGroup",
  component: RadioGroup,
  subcomponents: { RadioGroupItem },
  argTypes: {
    onValueChange: { action: "value changed" },
  },
} as Meta<typeof RadioGroup>

const Template: StoryFn<typeof RadioGroup> = (args) => (
  <RadioGroup {...args} defaultValue="comfortable">
    <div className="flex items-center space-x-2">
      <RadioGroupItem value="default" id="r1" />
      <Label htmlFor="r1">Default</Label>
    </div>
    <div className="flex items-center space-x-2">
      <RadioGroupItem value="comfortable" id="r2" />
      <Label htmlFor="r2">Comfortable</Label>
    </div>
    <div className="flex items-center space-x-2">
      <RadioGroupItem value="compact" id="r3" />
      <Label htmlFor="r3">Compact</Label>
    </div>
  </RadioGroup>
)

export const Default = Template.bind({})
Default.args = {}

export const WithDefaultValue = Template.bind({})
WithDefaultValue.args = {
  defaultValue: "option2",
}
