const tables = ["loans", "loan_events", "customer_events", "price_cents_btc", "sumsub_applicants", "credit_facility_events"]

envs.all.forEach((env) => {
  tables.forEach((table) => {
    declare({
      database: env.database,
      schema: env.importSchema,
      name: table,
      tags: ["lana"]
    })
  })
})
