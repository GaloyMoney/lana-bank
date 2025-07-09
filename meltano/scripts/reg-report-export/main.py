import sys
import pandas as pd
from os import environ
from datetime import datetime
from google.cloud import bigquery
from functions_framework import cloud_event


def get_current_report_df(table_name):
	try:
		client = bigquery.Client()
		table_id = f"{client.project}.{environ.get('DATASET_ID')}.{table_name}"
		print(table_id)

		sql_query = f"""
		SELECT *
		FROM `{table_id}`
		WHERE created_at = (SELECT MAX(created_at) FROM {table_id})
		ORDER BY 1
		;
		"""

		job = client.query(sql_query)
		results = job.result()
		df = results.to_dataframe()

		if len(df) <= 0:
			print(f"No report data from query='{sql_query}'")
			return None
		print(f"Successfully read {len(df)} rows from {table_id}")
		return df
	except Exception as e:
		print(f"Could not get report data from {table_id}")
		print(f"Exception: {e}")
		print(job.errors)
		return None

def exportRegulatoryReport(table_name):
	################################
	# get report list
	################################
	current_report_df = get_current_report_df(table_name)
	if (current_report_df is None) or len(current_report_df) <=0:
		print("Job failed")
		return

	# drop 'created_at' col
	report_created_at = current_report_df['created_at'][0]
	del current_report_df['created_at']

	################################
	# TODO: choose how and where to save report files, local | google drive | table
	################################

	# save report as local files
	current_report_df.to_xml(f'{table_name}.xml', index=False, root_name='data', row_name='row')
	current_report_df.to_csv(f'{table_name}.csv', index=False)

	# save report as files in google drive

	print("Job done")


if __name__ == "__main__":
	for arg in sys.argv[1:]:
		exportRegulatoryReport(arg)
