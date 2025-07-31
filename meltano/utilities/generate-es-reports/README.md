# generate-es-reports

`generate-es-reports` is a meltano utility to generate report files out of DW tables and store them.

You can run it with with `meltano invoke generate-es-reports run`.

Running with the environment variable `USE_LOCAL_FS` set to `1` will write to local filesystem instead of GCS. This is implemented only to make local development and debugging easier. You will find the generated reports in the same folder where the tool is.
