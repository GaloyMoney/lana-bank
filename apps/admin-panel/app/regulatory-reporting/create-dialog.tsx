import React from "react"
import { gql } from "@apollo/client"
import { toast } from "sonner"

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/primitive/dialog"
import { Button } from "@/components/primitive/button"
import { useReportCreateMutation } from "@/lib/graphql/generated"

gql`
  mutation ReportCreate {
    reportCreate {
      report {
        reportId
        createdAt
        lastError
        progress
      }
    }
  }
`

type ReportCreateDialogProps = {
  setOpenReportCreateDialog: (isOpen: boolean) => void
  openReportCreateDialog: boolean
  refetch: () => void
}

export const ReportCreateDialog: React.FC<ReportCreateDialogProps> = ({
  setOpenReportCreateDialog,
  openReportCreateDialog,
  refetch,
}) => {
  const [createReport, { loading }] = useReportCreateMutation()

  const handleCreateReport = async () => {
    try {
      const result = await createReport()
      if (result.data?.reportCreate?.report) {
        toast.success("Report creation started")
        refetch()
        setOpenReportCreateDialog(false)
      } else {
        throw new Error("No data returned from mutation")
      }
    } catch (error) {
      console.error("Error creating report:", error)
      toast.error("Failed to create report")
    }
  }

  return (
    <Dialog open={openReportCreateDialog} onOpenChange={setOpenReportCreateDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create New Report</DialogTitle>
          <DialogDescription>
            Are you sure you want to create a new report? This action will generate a
            regulatory report based on the latest financial data.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="ghost" onClick={() => setOpenReportCreateDialog(false)}>
            Cancel
          </Button>
          <Button onClick={handleCreateReport} loading={loading}>
            {loading ? "Creating..." : "Create Report"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
