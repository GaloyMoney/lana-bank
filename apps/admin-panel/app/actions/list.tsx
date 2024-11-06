import Link from "next/link"

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/primitive/table"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@/components/primitive/card"

const TABLE_ROWS = [
  { customer: "Jordan Michael", type: "Disbursement Approval", date: "2021-09-01" },
  { customer: "Alexa Liras", type: "Credit Facility Approval", date: "2021-09-01" },
  { customer: "Laurent Perrier", type: "Disbursement Approval", date: "2021-09-01" },
  { customer: "Michael Levi", type: "KYC Process", date: "2021-09-01" },
  { customer: "Richard Gain", type: "Disbursement Approval", date: "2021-09-01" },
]

const NUMBER_OF_ITEMS_IN_DASHBOARD = 3

type ListProps = {
  dashboard?: boolean
}

const List: React.FC<ListProps> = ({ dashboard = false }) => {
  const tableRows = dashboard
    ? TABLE_ROWS.slice(0, NUMBER_OF_ITEMS_IN_DASHBOARD)
    : TABLE_ROWS

  return (
    <Card>
      <CardHeader>
        <CardTitle>Pending Actions</CardTitle>
        <CardDescription>Approvals / Rejections waiting your way</CardDescription>
      </CardHeader>

      <CardContent>
        <div className="overflow-auto">
          <Table>
            {!dashboard && (
              <TableHeader>
                <TableRow>
                  <TableHead>Customer</TableHead>
                  <TableHead>Type</TableHead>
                  <TableHead>Date</TableHead>
                  <TableHead className="w-24"></TableHead>
                </TableRow>
              </TableHeader>
            )}
            <TableBody>
              {tableRows.map((data, idx) => (
                <TableRow key={idx}>
                  <TableCell className="font-medium">{data.customer}</TableCell>
                  <TableCell>{data.type}</TableCell>
                  <TableCell>{data.date}</TableCell>
                  <TableCell className="text-xs font-bold cursor-pointer">VIEW</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>

          {dashboard && (
            <div className="mt-4 flex items-center gap-2">
              <Link href="/actions" className="text-sm text-muted-foreground">
                ...{TABLE_ROWS.length - NUMBER_OF_ITEMS_IN_DASHBOARD} more
              </Link>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

export default List
