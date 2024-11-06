// src/components/Card/Card.stories.tsx
import React from "react"
import { Meta, StoryFn } from "@storybook/react"

import { Button } from "@/components/primitive/button"

import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from "@/components/primitive/card"

export default {
  title: "Components/Card",
  component: Card,
  subcomponents: {
    CardHeader,
    CardTitle,
    CardDescription,
    CardContent,
    CardFooter,
  },
  argTypes: {
    variant: {
      control: "select",
      options: ["primary", "secondary"],
      description: "Select the variant of card",
    },
    className: {
      control: "text",
      description: "Additional CSS classes to apply to the card",
    },
  },
} as Meta<typeof Card>

const Template: StoryFn<typeof Card> = (args) => <Card {...args} />

export const Default = Template.bind({})
Default.args = {
  children: (
    <>
      <CardHeader>
        <CardTitle>Title</CardTitle>
        <CardDescription>Description</CardDescription>
      </CardHeader>
      <CardContent>This is the content of the card.</CardContent>
      <CardFooter>This is the footer of the card.</CardFooter>
    </>
  ),
}

export const WithCustomStyles = Template.bind({})
WithCustomStyles.args = {
  className: "max-w-sm mx-auto",
  children: (
    <>
      <CardHeader>
        <CardTitle>Title</CardTitle>
        <CardDescription>Description</CardDescription>
      </CardHeader>
      <CardContent>This is the content of the card.</CardContent>
      <CardFooter>This is the footer of the card.</CardFooter>
    </>
  ),
}

export const WithComplexContent = Template.bind({})
WithComplexContent.args = {
  className: "max-w-lg mx-auto",
  children: (
    <>
      <CardHeader>
        <CardTitle>Title of Card with Complex Content</CardTitle>
        <CardDescription>
          More detailed explanation here with additional elements.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <ul>
          <li>Point 1</li>
          <li>Point 2</li>
          <li>Point 3</li>
        </ul>
      </CardContent>
      <CardFooter>
        <Button>Action</Button>
      </CardFooter>
    </>
  ),
}
