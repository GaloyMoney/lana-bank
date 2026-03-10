"use client"

import { usePathname } from "next/navigation"
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@lana/web/ui/sidebar"

import { CommandMenu } from "./command-menu"
import CreateButton, { CreateContextProvider } from "./create"

import { AppSidebar } from "@/components/app-sidebar"
import { HeaderUserMenu } from "@/components/header-user-menu"
import { RealtimePriceUpdates } from "@/components/realtime-price"
import { SearchAndCommand } from "@/components/search-and-command"

import { useCommandMenu } from "@/hooks/use-command-menu"
import { env } from "@/env"

export const AppLayout = ({ children }: Readonly<{ children: React.ReactNode }>) => {
  const appVersion = env.NEXT_PUBLIC_APP_VERSION
  const { open, setOpen, openCommandMenu } = useCommandMenu()
  const pathname = usePathname()
  const isJournalPage = pathname === "/journal"

  return (
    <CreateContextProvider>
      <SidebarProvider>
        <AppSidebar  appVersion={appVersion} />
        <SidebarInset className="md:peer-data-[variant=inset]:shadow-none border min-w-0">
          <CommandMenu open={open} onOpenChange={setOpen} />
          <header className="border-b">
            <div className="max-w-7xl mx-auto flex items-center py-2">
              <div className="flex items-center gap-2">
                <SidebarTrigger className="md:hidden" />
                <SearchAndCommand onOpenCommandPalette={openCommandMenu} />
              </div>
              <div className="flex items-center justify-end flex-1">
                <HeaderUserMenu />
              </div>
            </div>
          </header>
          <div
            className={
              isJournalPage
                ? "mx-auto pb-2 w-full min-w-0 flex-1 flex flex-col"
                : "container mx-auto pb-2 flex-1 flex flex-col"
            }
          >
            <div
              className={
                isJournalPage ? "w-full mx-auto min-w-0 flex flex-col flex-1" : "max-w-7xl w-full mx-auto flex flex-col flex-1"
              }
            >
              <RealtimePriceUpdates />
              <main className="flex-1 flex flex-col">{children}</main>
            </div>
          </div>
        </SidebarInset>
      </SidebarProvider>
    </CreateContextProvider>
  )
}
