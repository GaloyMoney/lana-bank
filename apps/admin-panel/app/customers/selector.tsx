import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/primitive/dialog"
import { Customer, useCustomersQuery } from "@/lib/graphql/generated"

import PaginatedTable, {
  Column,
  DEFAULT_PAGESIZE,
  PaginatedData,
} from "@/components/new/paginated-table"

type CustomerSelectorProps = {
  show: boolean
  setShow: React.Dispatch<React.SetStateAction<boolean>>
  onClose?: () => void
  setCustomer: (customer: Customer) => void
  title: string
}

const CustomerSelector: React.FC<CustomerSelectorProps> = ({
  show,
  setShow,
  onClose,
  setCustomer,
  title,
}) => {
  const { data, loading, fetchMore } = useCustomersQuery({
    variables: {
      first: DEFAULT_PAGESIZE,
    },
  })

  const closeCustomerSelector = () => {
    setShow(false)
  }

  return (
    <Dialog
      open={show}
      onOpenChange={() => {
        closeCustomerSelector()
        onClose && onClose()
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>
        <PaginatedTable<Customer>
          showHeader={false}
          columns={columns}
          data={data?.customers as PaginatedData<Customer>}
          loading={loading}
          fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
          pageSize={DEFAULT_PAGESIZE}
          onClick={(customer) => {
            setCustomer(customer)
            closeCustomerSelector()
          }}
        />
      </DialogContent>
    </Dialog>
  )
}

export default CustomerSelector

const columns: Column<Customer>[] = [{ key: "email", label: "Email" }]
