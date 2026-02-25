output "jobs_board_id" {
  value       = honeycombio_flexible_board.jobs.id
  description = "ID of the jobs dashboard"
}


output "credit_board_id" {
  value       = honeycombio_flexible_board.credit_board.id
  description = "ID of the credit dashboard"
}
