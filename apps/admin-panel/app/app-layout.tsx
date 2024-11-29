import CreateButton, { CreateContextProvider } from "./create"

import { RealtimePriceUpdates } from "@/components/realtime-price"

export const AppLayout = ({ children }: Readonly<{ children: React.ReactNode }>) => {
  return (
    <CreateContextProvider>
      <div className="container mx-auto p-2">
        <div className="max-w-7xl w-full mx-auto">
          <header className="flex justify-between items-center mb-2">
            <div className="font-semibold text-sm p-2 bg-secondary rounded-md">
              Welcome to Lana Bank
            </div>
            <CreateButton />
          </header>

          <RealtimePriceUpdates />
          <main>{children}</main>
        </div>
      </div>
    </CreateContextProvider>
  )
}
