// @ts-check
import { themes as prismThemes } from "prism-react-renderer";

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: "Lana Bank Documentation",
  tagline: "Technical documentation for the Lana Bank system",
  favicon: "img/favicon.ico",

  url: "https://lana-bank.github.io",
  baseUrl: "/",

  organizationName: "lana-bank",
  projectName: "lana-bank",

  onBrokenLinks: "throw",
  onBrokenMarkdownLinks: "warn",

  i18n: {
    defaultLocale: "en",
    locales: ["en", "es"],
    localeConfigs: {
      en: {
        label: "English",
        htmlLang: "en-US",
      },
      es: {
        label: "Español",
        htmlLang: "es-ES",
      },
    },
  },

  markdown: {
    mermaid: true,
  },

  themes: ["@docusaurus/theme-mermaid"],

  plugins: [
    [
      "@graphql-markdown/docusaurus",
      {
        id: "admin",
        schema: "../lana/admin-server/src/graphql/schema.graphql",
        rootPath: "./docs",
        baseURL: "api/admin",
        docOptions: {
          index: true,
          pagination: true,
        },
        loaders: {
          GraphQLFileLoader: "@graphql-tools/graphql-file-loader",
        },
      },
    ],
    [
      "@graphql-markdown/docusaurus",
      {
        id: "customer",
        schema: "../lana/customer-server/src/graphql/schema.graphql",
        rootPath: "./docs",
        baseURL: "api/customer",
        docOptions: {
          index: true,
          pagination: true,
        },
        loaders: {
          GraphQLFileLoader: "@graphql-tools/graphql-file-loader",
        },
      },
    ],
  ],

  presets: [
    [
      "classic",
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: "./sidebars.js",
          routeBasePath: "/",
        },
        blog: false,
        theme: {
          customCss: "./src/css/custom.css",
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      navbar: {
        title: "Lana Bank",
        items: [
          {
            type: "docSidebar",
            sidebarId: "docsSidebar",
            position: "left",
            label: "Documentation",
          },
          {
            to: "/api",
            label: "API Reference",
            position: "left",
          },
          {
            type: "localeDropdown",
            position: "right",
          },
          {
            href: "https://github.com/lana-bank/lana-bank",
            label: "GitHub",
            position: "right",
          },
        ],
      },
      footer: {
        style: "dark",
        links: [
          {
            title: "Docs",
            items: [
              {
                label: "Introduction",
                to: "/",
              },
              {
                label: "Credit",
                to: "/credit",
              },
              {
                label: "Accounting",
                to: "/accounting",
              },
            ],
          },
          {
            title: "API",
            items: [
              {
                label: "API Reference",
                to: "/api",
              },
              {
                label: "Admin API",
                to: "/api/admin",
              },
              {
                label: "Customer API",
                to: "/api/customer",
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} Lana Bank. Built with Docusaurus.`,
      },
      prism: {
        theme: prismThemes.github,
        darkTheme: prismThemes.dracula,
        additionalLanguages: ["rust", "toml", "bash", "graphql"],
      },
      mermaid: {
        theme: { light: "neutral", dark: "dark" },
      },
    }),
};

export default config;
