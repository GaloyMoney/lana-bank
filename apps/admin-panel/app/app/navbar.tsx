import Avatar from "./avatar"
import { Logo } from "@/components"

const NavBar = () => {
  return (
    <div className="flex flex-col h-full min-w-[240px] items-start">
      <div className="flex justify-between items-center w-full p-[20px]">
        <Logo />
        <Avatar />
      </div>
    </div>
  )
}

export default NavBar
