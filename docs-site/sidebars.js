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
      label: "Technical Documentation",
      link: {
        type: "doc",
        id: "technical-documentation/index",
      },
      items: [
        {
          type: "category",
          label: "Customer Management",
          link: {
            type: "doc",
            id: "technical-documentation/customers/index",
          },
          items: [
            "technical-documentation/customers/onboarding",
            "technical-documentation/customers/documents",
          ],
        },
        {
          type: "category",
          label: "Deposits and Withdrawals",
          link: {
            type: "doc",
            id: "technical-documentation/deposits/index",
          },
          items: [
            "technical-documentation/deposits/operations",
          ],
        },
        {
          type: "category",
          label: "Credit Management",
          link: {
            type: "doc",
            id: "technical-documentation/credit/index",
          },
          items: [
            "technical-documentation/credit/facility",
            "technical-documentation/credit/disbursal",
            "technical-documentation/credit/obligation",
            "technical-documentation/credit/payment",
            "technical-documentation/credit/terms",
            "technical-documentation/credit/interest-process",
          ],
        },
        {
          type: "category",
          label: "Accounting",
          link: {
            type: "doc",
            id: "technical-documentation/accounting/index",
          },
          items: [
            "technical-documentation/accounting/closing",
            "technical-documentation/accounting/fiscal-year",
          ],
        },
        {
          type: "category",
          label: "Approvals and Governance",
          link: {
            type: "doc",
            id: "technical-documentation/approvals/index",
          },
          items: [
            "technical-documentation/approvals/committees",
            "technical-documentation/approvals/policies",
          ],
        },
        {
          type: "category",
          label: "Financial Reports",
          link: {
            type: "doc",
            id: "technical-documentation/reporting/index",
          },
          items: [
            "technical-documentation/reporting/financial-reports",
          ],
        },
      ],
    },
    {
      type: "category",
      label: "For Internal Developers",
      link: {
        type: "doc",
        id: "for-internal-developers/index",
      },
      items: [
        "for-internal-developers/local-development",
        "for-internal-developers/authentication-local",
        "for-internal-developers/authorization",
        "for-internal-developers/graphql-development",
        "for-internal-developers/configuration",
        {
          type: "category",
          label: "Frontend Applications",
          link: {
            type: "doc",
            id: "for-internal-developers/frontend/index",
          },
          items: [
            "for-internal-developers/frontend/admin-panel",
            "for-internal-developers/frontend/customer-portal",
            "for-internal-developers/frontend/shared-components",
            "for-internal-developers/frontend/credit-ui",
          ],
        },
        {
          type: "category",
          label: "Domain Architecture",
          items: [
            "for-internal-developers/domain-services",
            "for-internal-developers/event-system",
            "for-internal-developers/background-jobs",
            "for-internal-developers/cala-ledger-integration",
            "for-internal-developers/custody-portfolio",
          ],
        },
        {
          type: "category",
          label: "Infrastructure",
          items: [
            "for-internal-developers/infrastructure-services",
            "for-internal-developers/observability",
            "for-internal-developers/audit-system",
          ],
        },
      ],
    },
    {
      type: "category",
      label: "For External Developers",
      link: {
        type: "doc",
        id: "for-external-developers/index",
      },
      items: [
        "for-external-developers/quickstart",
        "for-external-developers/authentication",
        "for-external-developers/graphql-integration",
        "for-external-developers/webhooks",
      ],
    },
    {
      type: "category",
      label: "APIs",
      link: {
        type: "doc",
        id: "apis/index",
      },
      items: [
        "apis/admin-api/api-reference",
        "apis/customer-api/api-reference",
        "apis/events/events",
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
            "for-platform-engineers/functional-architecture",
            "for-platform-engineers/authentication-architecture",
          ],
        },
        "for-platform-engineers/data-pipelines",
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
