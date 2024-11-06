"use client"

import { toast } from "sonner"

type IDProps = {
  id: string
  type?: string
}

const ID: React.FC<IDProps> = ({ id, type }) => {
  const copyID = () => {
    navigator.clipboard.writeText(id)
    toast.success((type ? type + " " : "") + "ID is copied to clipboard")
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
