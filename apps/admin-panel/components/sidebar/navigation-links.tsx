"use client"
import Link from "next/link"
import { usePathname } from "next/navigation"

import { IoPerson, IoReceipt, IoCash } from "react-icons/io5"
import { MdAccountTree } from "react-icons/md"

const navLinks = [
  { href: "/loan", label: "Loan", icon: IoReceipt },
  { href: "/user", label: "Users", icon: IoPerson },
  { href: "/trial-balance", label: "Trial Balance", icon: IoCash },
  { href: "/chart-of-accounts", label: "Chart of Accounts", icon: MdAccountTree },
]

const NavigationLinks = () => {
  const pathname = usePathname()
  return (
    <nav className="flex flex-col gap-6 text-textColor-secondary">
      {navLinks.map((link, index) => (
        <Link
          key={index}
          href={link.href}
          prefetch={false}
          className={`hover:text-textColor-primary ${pathname === link.href && "text-primary"}`}
        >
          <div className="flex items-center gap-4 rounded-md">
            <link.icon className="w-5 h-5" />
            {link.label}
          </div>
        </Link>
      ))}
    </nav>
  )
}

export { NavigationLinks }
