import { Button } from "@/components"
import Link from "next/link"

type ErrorSearchParams = {
  error?: string
}

type ErrorProps = {
  searchParams?: ErrorSearchParams
}

const Error: React.FC<ErrorProps> = ({ searchParams }) => (
  <>
    <h1 className="text-heading-h3">Access Denied</h1>
    <div className="space-y-[10px]">
      <div className="text-title-md !text-error">Oops, we could not sign you in</div>
      <div className="text-body-md !text-error">
        Please recheck your credentials and try again. Repeated attempts with wrong email
        might ban your IP from the system.
      </div>
      <div className="text-title-sm !text-error !mb-5">
        Reason: {searchParams?.error || "Unknown"}
      </div>
      <Link href="/auth/login">
        <Button title="Back to login" />
      </Link>
    </div>
  </>
)

export default Error
