import click

from generate_es_reports.service import run_report_batch


@click.group()
def cli():
    pass


@cli.command
def run():
    run_report_batch()
