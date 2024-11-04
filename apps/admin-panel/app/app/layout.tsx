import { HiPlus, HiSearch } from "react-icons/hi"

import { Button, Input } from "@/components"
import NavBar from "./navbar"

const AppLayout: React.FC<React.PropsWithChildren> = ({ children }) => {
  return (
    <div className="bg-soft h-full w-full flex flex-col md:flex-row">
      <NavBar />
      <div className="flex-1 pt-[72px] md:pt-[10px] overflow-auto no-scrollbar p-[10px]">
        <div className="p-[10px] border rounded-md w-full">
          <div className="md:flex gap-2 hidden">
            <Input
              type="text"
              placeholder="Search for Customer, Credit Facility or Menu Items"
              leftNode={<HiSearch className="text-placeholder" />}
              rightNode={<div className="!text-placeholder text-body-sm">⌘ + K or /</div>}
            />
            <Button
              size="md"
              title="Create"
              icon={<HiPlus className="text-lg" />}
              className="py-3 px-6 w-36"
            />
          </div>
          <div className="">{children}</div>
        </div>
      </div>
    </div>
  )
}

export default AppLayout
