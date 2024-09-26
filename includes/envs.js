const dev = {
  name: "volcano-dev",
  database: "lava-dev-436720",
  importSchema: dataform.projectConfig.vars.devUser + "_dataset",
}

const staging = {
  name: "volcano-staging",
  database: "volcano-staging",
  importSchema: "volcano_staging_dataset",
}

const paramsByName = {
  [dev.name]: dev,
  [staging.name]: staging,
}

module.exports = {
  all: [dev, staging],
  current: dataform.projectConfig.vars.executionEnv,
  currentDatabase: paramsByName[dataform.projectConfig.vars.executionEnv].database,
  currentImportSchema: paramsByName[dataform.projectConfig.vars.executionEnv].importSchema
}
