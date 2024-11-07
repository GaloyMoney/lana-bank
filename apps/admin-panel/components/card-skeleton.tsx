import { Skeleton } from "./primitive/skeleton"

export const CardSkeleton = () => (
  <div className="w-full p-6 border rounded-lg">
    <div className="space-y-2">
      <div className="flex items-end gap-2">
        <Skeleton className="h-8 w-16" />
        <Skeleton className="h-6 w-12" />
      </div>
      <Skeleton className="h-6 w-32" />
      <Skeleton className="h-4 w-48" />
    </div>
    <div className="mt-4">
      <Skeleton className="h-9 w-28" />
    </div>
  </div>
)
