import type { Config } from "tailwindcss"

const config: Config = {
  darkMode: ["class"],
  content: ["app/**/*.{ts,tsx}", "components/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: "var(--primary)",
          foreground: "var(--primary-foreground)",
        },
        secondary: {
          DEFAULT: "var(--secondary)",
          foreground: "var(--secondary-foreground)",
        },
        textColor: {
          primary: "var(--text-primary)",
          secondary: "var(--text-secondary)",
        },
        button: {
          text: {
            DEFAULT: "var(--button-text-primary)",
            secondary: "var(--button-text-secondary)",
          },
        },
        input: {
          text: {
            DEFAULT: "var(--input-text-primary)",
          },
        },
        success: "var(--green)",
        warning: "var(--warning)",
        destructive: "var(--error)",
      },
      boxShadow: {
        glow: "0 0 8px var(--primary)",
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
        "caret-blink": "caret-blink 1.25s ease-out infinite",
      },
      borderColor: {
        DEFAULT: "var(--border-color)",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
}
export default config
