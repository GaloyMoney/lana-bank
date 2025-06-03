import type { Meta, StoryObj } from "@storybook/nextjs"
import { MockedProvider } from "@apollo/client/testing"

import TermsTemplatePage from "./page"

import faker from "@/.storybook/faker"

import { TermsTemplateDocument } from "@/lib/graphql/generated"
import { mockTermsTemplate } from "@/lib/graphql/generated/mocks"

interface TermsTemplateStoryArgs {
  name: string
}

const DEFAULT_ARGS: TermsTemplateStoryArgs = {
  name: "Medium Risk",
}

const createMocks = (args: TermsTemplateStoryArgs, templateId: string) => [
  {
    request: {
      query: TermsTemplateDocument,
      variables: { id: templateId },
    },
    result: {
      data: {
        termsTemplate: mockTermsTemplate({
          name: args.name,
        }),
      },
    },
  },
]

const TermsTemplateStory = (args: TermsTemplateStoryArgs) => {
  const templateId = faker.string.uuid()
  const mocks = createMocks(args, templateId)

  return (
    <MockedProvider mocks={mocks} addTypename={false} key={JSON.stringify(args)}>
      <TermsTemplatePage params={Promise.resolve({ "terms-template-id": templateId })} />
    </MockedProvider>
  )
}

const LoadingStory = () => {
  const templateId = faker.string.uuid()
  const mocks = [
    {
      request: {
        query: TermsTemplateDocument,
        variables: { id: templateId },
      },
      delay: Infinity,
    },
  ]

  return (
    <MockedProvider mocks={mocks} addTypename={false}>
      <TermsTemplatePage params={Promise.resolve({ "terms-template-id": templateId })} />
    </MockedProvider>
  )
}

const meta: Meta<typeof TermsTemplateStory> = {
  title: "Pages/TermsTemplates/TermsTemplate/Details",
  component: TermsTemplateStory,
  parameters: { layout: "fullscreen", nextjs: { appDirectory: true } },
  argTypes: {
    name: {
      control: "text",
      description: "Template name",
    },
  },
}

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = { args: DEFAULT_ARGS }

export const Loading: Story = {
  args: DEFAULT_ARGS,
  render: LoadingStory,
  parameters: {
    nextjs: {
      navigation: {
        pathname: "/terms-templates/[terms-template-id]",
      },
    },
  },
}
