"use client"

import { toast } from "sonner"

type IDProps = {
  id: string
}

const ID: React.FC<IDProps> = ({ id }) => {
  const copyID = () => {
    navigator.clipboard.writeText(id)
    toast.success("ID copied to clipboard")
  }

  return (
    <div className="text-[10px]">
      <span className="text-mono font-light">{id.slice(0, 4)}...</span>
      <span className="text-primary cursor-pointer" onClick={copyID}>
        Copy ID
      </span>
    </div>
  )
}

export default ID
