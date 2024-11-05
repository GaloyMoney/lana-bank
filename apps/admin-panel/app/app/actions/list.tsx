import Link from "next/link"

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
    <div className="bg-page rounded-md p-[10px] flex flex-col gap-1 w-full border">
      <div className="text-title-md">Pending Actions</div>
      <div className={`!text-body text-body-sm ${dashboard && "mb-2"}`}>
        Approvals / Rejections waiting your way
      </div>
      <div className="overflow-auto h-full w-full">
        <table className="w-full min-w-max table-auto text-left">
          {!dashboard && (
            <thead>
              <tr>
                <th className="pt-4 pb-2 text-heading text-title-sm">Customer</th>
                <th className="pt-4 pb-2 text-heading text-title-sm">Type</th>
                <th className="pt-4 pb-2 text-heading text-title-sm">Date</th>
                <th className="pt-4 pb-2 text-heading text-title-sm"></th>
              </tr>
            </thead>
          )}
          <tbody>
            {tableRows.map((data, idx) => (
              <tr key={idx}>
                <td className="text-body-md p-1">{data.customer}</td>
                <td className="text-body-md p-1">{data.type}</td>
                <td className="text-body-md p-1">{data.date}</td>
                <td className="!text-action text-title-sm cursor-pointer pt-1">VIEW</td>
              </tr>
            ))}
          </tbody>
        </table>
        {dashboard && (
          <div className="mt-2 flex items-center gap-2">
            <span className="text-body-md">
              ...{TABLE_ROWS.length - NUMBER_OF_ITEMS_IN_DASHBOARD} more
            </span>
            <Link
              href="/app/actions"
              className="uppercase !text-action text-title-sm cursor-pointer"
            >
              View All
            </Link>
          </div>
        )}
      </div>
    </div>
  )
}

export default List
