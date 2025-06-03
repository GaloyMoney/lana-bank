import type { StorybookConfig } from "@storybook/nextjs"

const config: StorybookConfig = {
  stories: ["../**/*.mdx", "../**/*.stories.@(js|jsx|mjs|ts|tsx)"],
  features: {
    experimentalRSC: true,
  },
  addons: [
    "@storybook/addon-onboarding",
    "@storybook/addon-links",
    "@chromatic-com/storybook",
    "@storybook/addon-postcss",
    "@storybook/addon-docs"
  ],
  framework: {
    name: "@storybook/nextjs",
    options: {},
  },
  webpackFinal: async (config) => {
    const imageRule = config.module?.rules?.find((rule) => {
      const test = (rule as { test: RegExp }).test
      if (!test) {
        return false
      }
      return test.test(".svg")
    }) as { [key: string]: any }
    imageRule.exclude = /\.svg$/
    config.module?.rules?.push({
      test: /\.svg$/,
      use: ["@svgr/webpack"],
    })
    return config
  },
}
export default config
