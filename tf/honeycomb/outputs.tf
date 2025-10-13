output "jobs_board_id" {
  value       = honeycombio_flexible_board.jobs.id
  description = "ID of the jobs dashboard"
}

output "event_jobs_board_id" {
  value       = honeycombio_flexible_board.event_jobs.id
  description = "ID of the jobs dashboard"
}
