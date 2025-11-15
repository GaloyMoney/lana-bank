import pandas as pd

import dagster as dg


def iris_dataset_size(context: dg.AssetExecutionContext) -> None:
    """Asset that loads and analyzes the Iris dataset with OpenTelemetry tracing"""

    df = pd.read_csv(
        "https://docs.dagster.io/assets/iris.csv",
        names=[
            "sepal_length_cm",
            "sepal_width_cm",
            "petal_length_cm",
            "petal_width_cm",
            "species",
        ],
    )

    row_count = df.shape[0]
    col_count = df.shape[1]

    context.log.info(
        f"📊 Final result: Loaded {row_count} data points with {col_count} features"
    )
