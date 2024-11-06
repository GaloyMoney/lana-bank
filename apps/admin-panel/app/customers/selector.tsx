import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/primitive/dialog"
import { Customer } from "@/lib/graphql/generated"

type CustomerSelectorProps = {
  show: boolean
  setShow: React.Dispatch<React.SetStateAction<boolean>>
  onClose?: () => void
  setCustomer: (customer: Customer) => void
  title: string
  description: string
}

const CustomerSelector: React.FC<CustomerSelectorProps> = ({
  show,
  setShow,
  onClose,
  setCustomer,
  title,
  description,
}) => {
  return (
    <Dialog
      open={show}
      onOpenChange={() => {
        setShow(false)
        onClose && onClose()
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}r</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
      </DialogContent>
    </Dialog>
  )
}

export default CustomerSelector
