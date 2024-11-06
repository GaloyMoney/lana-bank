import {
  Dialog,
  DialogContent,
  DialogDescription,
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
  const { data, fetchMore } = useCustomersQuery({
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
          columns={columns}
          data={data?.customers as PaginatedData<Customer>}
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
