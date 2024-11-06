/** @type {import('next').NextConfig} */
const nextConfig = {
  webpack: (config) => {
    config.module.rules.push({
      test: /\.svg$/i,
      use: ["@svgr/webpack"],
    })
    return config
  },
  experimental: {
    instrumentationHook: true,
  },
  output: "standalone",
  basePath: process.env.BASE_PATH,
}

export default nextConfig
