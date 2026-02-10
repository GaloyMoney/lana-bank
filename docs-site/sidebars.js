/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  docsSidebar: [
    "intro",
    {
      type: "category",
      label: "Getting Started",
      link: {
        type: "doc",
        id: "getting-started/index",
      },
      items: [],
    },
    {
      type: "category",
      label: "For Developers",
      link: {
        type: "doc",
        id: "for-developers/index",
      },
      items: [
        "for-developers/quickstart",
        "for-developers/authentication",
        "for-developers/graphql-integration",
        "for-developers/webhooks",
        {
          type: "category",
          label: "Frontend Applications",
          link: {
            type: "doc",
            id: "for-developers/frontend/index",
          },
          items: [
            "for-developers/frontend/admin-panel",
            "for-developers/frontend/customer-portal",
            "for-developers/frontend/shared-components",
            "for-developers/frontend/credit-ui",
          ],
        },
        "for-developers/admin-api/api-reference",
        "for-developers/customer-api/api-reference",
        "for-developers/events/events",
      ],
    },
    {
      type: "category",
      label: "For Operators",
      link: {
        type: "doc",
        id: "for-operators/index",
      },
      items: [
        {
          type: "category",
          label: "Customer Management",
          link: {
            type: "doc",
            id: "for-operators/customers/index",
          },
          items: [
            "for-operators/customers/onboarding",
            "for-operators/customers/documents",
          ],
        },
        {
          type: "category",
          label: "Deposits and Withdrawals",
          link: {
            type: "doc",
            id: "for-operators/deposits/index",
          },
          items: [
            "for-operators/deposits/operations",
          ],
        },
        {
          type: "category",
          label: "Credit Management",
          link: {
            type: "doc",
            id: "for-operators/credit/index",
          },
          items: [
            "for-operators/credit/facility",
            "for-operators/credit/disbursal",
            "for-operators/credit/obligation",
            "for-operators/credit/payment",
            "for-operators/credit/terms",
            "for-operators/credit/interest-process",
          ],
        },
        {
          type: "category",
          label: "Accounting",
          link: {
            type: "doc",
            id: "for-operators/accounting/index",
          },
          items: [
            "for-operators/accounting/closing",
            "for-operators/accounting/fiscal-year",
          ],
        },
        {
          type: "category",
          label: "Approvals and Governance",
          link: {
            type: "doc",
            id: "for-operators/approvals/index",
          },
          items: [
            "for-operators/approvals/committees",
            "for-operators/approvals/policies",
          ],
        },
        {
          type: "category",
          label: "Financial Reports",
          link: {
            type: "doc",
            id: "for-operators/reporting/index",
          },
          items: [
            "for-operators/reporting/financial-reports",
          ],
        },
        {
          type: "category",
          label: "Configuration",
          link: {
            type: "doc",
            id: "for-operators/configuration/index",
          },
          items: [],
        },
      ],
    },
    {
      type: "category",
      label: "For Platform Engineers",
      link: {
        type: "doc",
        id: "for-platform-engineers/index",
      },
      items: [
        {
          type: "category",
          label: "System Architecture",
          items: [
            "for-platform-engineers/system-architecture",
            "for-platform-engineers/domain-services",
            "for-platform-engineers/functional-architecture",
          ],
        },
        {
          type: "category",
          label: "Technical Infrastructure",
          items: [
            "for-platform-engineers/authentication-architecture",
            "for-platform-engineers/event-system",
            "for-platform-engineers/background-jobs",
            "for-platform-engineers/infrastructure-services",
            "for-platform-engineers/observability",
            "for-platform-engineers/audit-system",
          ],
        },
        {
          type: "category",
          label: "Integrations",
          items: [
            "for-platform-engineers/cala-ledger-integration",
            "for-platform-engineers/custody-portfolio",
            "for-platform-engineers/data-pipelines",
          ],
        },
        {
          type: "category",
          label: "Data Models (ERDs)",
          link: {
            type: "doc",
            id: "for-platform-engineers/erds/index",
          },
          items: [
            "for-platform-engineers/erds/cala",
            "for-platform-engineers/erds/lana",
          ],
        },
        {
          type: "category",
          label: "Deployment and Operations",
          link: {
            type: "doc",
            id: "for-platform-engineers/deployment/index",
          },
          items: [
            "for-platform-engineers/deployment/build-system",
            "for-platform-engineers/deployment/development-environment",
            "for-platform-engineers/deployment/testing-strategy",
            "for-platform-engineers/deployment/ci-cd",
          ],
        },
      ],
    },
    {
      type: "category",
      label: "Reference",
      items: [
        "reference/glossary",
        "reference/changelog",
      ],
    },
  ],
};

export default sidebars;
