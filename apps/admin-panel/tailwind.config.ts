import type { Config } from "tailwindcss"
import sharedConfig from "@lana/web/tailwind.config"

const config = {
  presets: [sharedConfig],
  content: [
    "./app/**/*.{ts,tsx}",
    "./components/**/*.{ts,tsx}",
    "../shared-web/src/**/*.{ts,tsx}",
  ],
} satisfies Config

export default config
