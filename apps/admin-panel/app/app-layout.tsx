"use client"

import { SidebarInset, SidebarProvider, SidebarTrigger } from "@lana/web/ui/sidebar"

import { CommandMenu } from "./command-menu"
import CreateButton, { CreateContextProvider } from "./create"
import { DynamicBreadcrumb } from "./dynamic-breadcrumb"

import { AppSidebar } from "@/components/app-sidebar"
import { RealtimePriceUpdates } from "@/components/realtime-price"
import { SearchAndCommand } from "@/components/search-and-command"

import { useCommandMenu } from "@/hooks/use-command-menu"

import { env } from "@/env"

export const AppLayout = ({ children }: Readonly<{ children: React.ReactNode }>) => {
  const appVersion = env.NEXT_PUBLIC_APP_VERSION
  const { open, setOpen, openCommandMenu } = useCommandMenu()

  return (
    <CreateContextProvider>
      <SidebarProvider>
        <AppSidebar appVersion={appVersion} />
        <SidebarInset className="min-h-screen md:peer-data-[variant=inset]:shadow-none border">
          <CommandMenu open={open} onOpenChange={setOpen} />
          <div className="container mx-auto p-2">
            <div className="max-w-7xl w-full mx-auto">
              <header className="flex justify-between items-center mb-2 align-middle">
                <div className="flex items-center gap-2">
                  <SidebarTrigger className="md:hidden" />
                  <DynamicBreadcrumb />
                </div>
                <div className="flex items-center gap-2 flex-1 justify-end">
                  <SearchAndCommand onOpenCommandPalette={openCommandMenu} />
                  <CreateButton />
                </div>
              </header>
              <RealtimePriceUpdates />
              <main>{children}</main>
            </div>
          </div>
        </SidebarInset>
      </SidebarProvider>
    </CreateContextProvider>
  )
}
