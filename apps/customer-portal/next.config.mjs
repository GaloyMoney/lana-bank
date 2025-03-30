/** @type {import('next').NextConfig} */
const nextConfig = {
  webpack: (config) => {
    config.module.rules.push({
      test: /\.svg$/i,
      use: ["@svgr/webpack"],
    })
    return config
  },
  transpilePackages: ["@lana/web"],
  output: "standalone",
  basePath: process.env.NEXT_PUBLIC_BASE_PATH,
  experimental: {
    turbo: {
      enabled: true,
      rules: {
        "**/*.svg": {
          loaders: ["@svgr/webpack"],
          as: "default",
        },
      }
    }
  }
}

export default nextConfig
