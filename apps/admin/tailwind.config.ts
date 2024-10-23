import type { Config } from "tailwindcss"

import withMT from "@material-tailwind/react/utils/withMT"

const config: Config = {
  content: [
    "./pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      fontFamily: {
        inter: ["var(--font-inter)", "sans-serif"],
        helvetica: ["var(--font-helvetica)", "sans-serif"],
      },
      colors: {
        "primary": "#3E54FB",
        "secondary": "#363849",
        "white": "#FFFFFF",
        "background": "#F6F8FA",
        "primary-bg": "#EBEDFF",
        "warning": "#FA9A20",
        "warning-bg": "#FFF6EB",
        "success": "#2AAF96",
        "success-bg": "#EFFBF9",
        "grey": {
          0: "#212336",
        },
      },
    },
  },
  plugins: [],
}

export default withMT(config)
