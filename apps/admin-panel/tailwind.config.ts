import type { Config } from "tailwindcss"
import withMT from "@material-tailwind/react/utils/withMT"

import { aliasColors } from "./lib/ui/primitives"

const config: Config = {
  content: [
    "./pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: aliasColors,
      textColor: ({ theme }) => ({
        "heading": theme("colors.neutral.900"),
        "body": theme("colors.neutral.800"),
        "primary": theme("colors.primary"),
        "action": theme("colors.primary"),
        "action-hover": theme("colors.primary.600"),
        "disabled": theme("colors.neutral.400"),
        "success": theme("colors.success"),
        "warning": theme("colors.warning"),
        "error": theme("colors.error"),
        "info": theme("colors.info"),
        "on-action": theme("colors.neutral.white"),
        "on-disabled": theme("colors.neutral.700"),
        "placeholder": theme("colors.blue-gray.300"),
      }),
      backgroundColor: ({ theme }) => ({
        "soft": theme("colors.neutral.50"),
        "page": theme("colors.neutral.white"),
        "primary": theme("colors.primary"),
        "success": theme("colors.success.50"),
        "warning": theme("colors.warning.50"),
        "error": theme("colors.error.50"),
        "info": theme("colors.info.50"),
        "action": theme("colors.primary"),
        "action-hover": theme("colors.primary.600"),
        "action-secondary": theme("colors.neutral.900"),
        "action-secondary-hover": theme("colors.neutral.200"),
        "disabled": theme("colors.blue-gray.50"),
      }),
      borderColor: ({ theme }) => ({
        "default": theme("colors.neutral.200"),
        "primary": theme("colors.primary"),
        "action-hover": theme("colors.primary.600"),
        "success": theme("colors.success"),
        "warning": theme("colors.warning"),
        "error": theme("colors.error"),
        "info": theme("colors.info"),
        "input": theme("colors.blue-gray.100"),
        "input-focus": theme("colors.blue-gray.700"),
      }),
      borderRadius: {
        sm: "4px",
        md: "10px",
      },
    },
  },
  plugins: [],
}

export default withMT(config)
