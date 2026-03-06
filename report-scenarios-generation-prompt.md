dagster/generate_es_reports contains a Python package that generates file reports out of tables in Bigquery. These reports are related to El Salvador banking regulations. You can find the text for norms in notes/norms.

These bigquery tables get generated via the dbt project at dagster/src/dbt_lana_dw. Most of the data comes from the lana sources. Lana is the application that exists in this repository. It uses event sourcing, with the events being stored in Postgres. The dbt sources are tables of this postgres database.

We're interested in running some seeding efforts to provide interesting, realistic data for data pipeline development purposes. The goal is to have data in the postgres database that is complete features wise, and relevant to assess the correctness of the final file report for the norms.

I want you to do the following:
- Read NRSF-03 norm.
- Read the NRSF-03 reports that exist within dagster/generate_es_reports.
- Map out the whole dependency chain all the way to internals of Lana.
- Then, taking into account the reporting needs described by the norm, produce a scenarios proposal. This should be a markdown file where you describe:
  - What should happen in Lana to trigger proper events that generate realistic, business relevant data.
  - Which events you expect to see generated from this.
  - Justify which parts of the reporting regulation calls for them.

Store the output in a document called report-scenarios.md.
