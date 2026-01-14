/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  docsSidebar: [
    "intro",
    {
      type: "category",
      label: "Credit",
      link: {
        type: "doc",
        id: "credit/index",
      },
      items: [
        "credit/facility",
        "credit/disbursal",
        "credit/obligation",
        "credit/payment",
        "credit/terms",
        "credit/interest-process",
      ],
    },
    {
      type: "category",
      label: "Accounting",
      link: {
        type: "doc",
        id: "accounting/index",
      },
      items: ["accounting/closing", "accounting/fiscal-year"],
    },
    {
      type: "category",
      label: "ERDs",
      link: {
        type: "doc",
        id: "erds/index",
      },
      items: ["erds/cala", "erds/lana"],
    },
  ],
};

export default sidebars;
