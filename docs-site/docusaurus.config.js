// @ts-check
import { themes as prismThemes } from "prism-react-renderer";

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: "Lana Bank Documentation",
  tagline: "Technical documentation for the Lana Bank system",
  favicon: "img/favicon.ico",

  url: "https://galoymoney.github.io",
  baseUrl: "/lana-bank/",

  organizationName: "galoymoney",
  projectName: "lana-bank",

  onBrokenLinks: "throw",

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
    hooks: {
      onBrokenMarkdownLinks: "warn",
    },
  },

  themes: ["@docusaurus/theme-mermaid"],

  plugins: [
    [
      "@graphql-markdown/docusaurus",
      {
        id: "admin",
        schema: "../lana/admin-server/src/graphql/schema.graphql",
        rootPath: "./docs",
        baseURL: "for-developers/admin-api",
        docOptions: {
          index: true,
          pagination: true,
        },
        loaders: {
          GraphQLFileLoader: "@graphql-tools/graphql-file-loader",
        },
        sidebar: {
          sidebarId: "docsSidebar",
        },
      },
    ],
    [
      "@graphql-markdown/docusaurus",
      {
        id: "customer",
        schema: "../lana/customer-server/src/graphql/schema.graphql",
        rootPath: "./docs",
        baseURL: "for-developers/customer-api",
        docOptions: {
          index: true,
          pagination: true,
        },
        loaders: {
          GraphQLFileLoader: "@graphql-tools/graphql-file-loader",
        },
        sidebar: {
          sidebarId: "docsSidebar",
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
          versions: {
            current: {
              label: "Next",
            },
          },
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
            to: "/for-developers",
            label: "API Reference",
            position: "left",
          },
          {
            type: "docsVersionDropdown",
            position: "right",
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
            title: "Documentation",
            items: [
              {
                label: "Getting Started",
                to: "/getting-started",
              },
              {
                label: "For Developers",
                to: "/for-developers",
              },
              {
                label: "For Operators",
                to: "/for-operators",
              },
              {
                label: "For Platform Engineers",
                to: "/for-platform-engineers",
              },
            ],
          },
          {
            title: "API Reference",
            items: [
              {
                label: "Admin API",
                to: "/for-developers/admin-api",
              },
              {
                label: "Customer API",
                to: "/for-developers/customer-api",
              },
              {
                label: "Domain Events",
                to: "/for-developers/events",
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
      algolia: {
        appId: "61TV6H03QM",
        apiKey: "7103f73bed9ebf4c96e326e33ecf4a01",
        indexName: "Lana Bank documentation",
        contextualSearch: true,
      },
    }),
};

export default config;
