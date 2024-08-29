const tables = ["loans", "loan_events"]

  tables.forEach((table) => {
    declare({
      database: "cala-enterprise",
      schema: "jireva_lava_dev_dataset",
      name: table,
      tags: ["lava"]
    })
  })
