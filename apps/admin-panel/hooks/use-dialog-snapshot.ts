import { useEffect, useState } from "react"

export function useDialogSnapshot<T>(value: T, open: boolean): T {
  const [snapshot, setSnapshot] = useState(value)

  useEffect(() => {
    if (open) {
      setSnapshot(value)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open])

  return snapshot
}
