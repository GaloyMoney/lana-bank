import { IoMenu } from "react-icons/io5"
import Link from "next/link"

import { LavaBankIcon } from "../icon"

import { NavigationLinks } from "./navigation-links"

import { Sheet, SheetTrigger, SheetContent } from "@/components/primitive/sheet"
import { Button } from "@/components/primitive/button"

export function SideBar() {
  return (
    <>
      <div className="hidden md:block bg-primary-foreground border-r border-secondary-foreground w-64">
        <div className="flex flex-col h-full">
          <div className="flex items-center justify-between p-6">
            <LavaBankIcon />
          </div>
          <div className="flex flex-col justify-between h-full">
            <div className="flex flex-col  ml-6 mt-4">
              <NavigationLinks />
            </div>
            <div className="flex justify-center items-center p-4 border-t border-secondary-foreground">
              <Link href="/profile">
                <Button variant="secondary" className="w-56">
                  <div className="flex gap-2 items-center">
                    <p>Profile</p>
                  </div>
                </Button>
              </Link>
            </div>
          </div>
        </div>
      </div>

      <div className="md:hidden">
        <Sheet>
          <SheetTrigger asChild>
            <div className="flex items-center p-4">
              <IoMenu className="w-6 h-6" />
            </div>
          </SheetTrigger>
          <SheetContent side="left" className="flex flex-col justify-between h-full">
            <div className="p-4">
              <h1 className="text-xl font-bold mb-4">LAVA BANK</h1>
              <NavigationLinks />
            </div>
            <div className="flex justify-center items-center p-4 border-t border-secondary-foreground">
              <Link href="/profile">Profile</Link>
            </div>
          </SheetContent>
        </Sheet>
      </div>
    </>
  )
}
