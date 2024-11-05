"use client"

import Carausel from "./carausel"

import { Logo } from "@/components"

const AuthLayout: React.FC<React.PropsWithChildren> = ({ children }) => (
  <div className="flex flex-col lg:flex-row p-[40px] lg:space-x-[10px] space-y-[10px] lg:space-y-0 h-full">
    <div className="hidden lg:block bg-primary lg:max-w-[426px] py-20 rounded-md">
      <Carausel />
    </div>
    <div className="lg:hidden bg-primary h-[20px] w-full rounded-sm"></div>
    <div className="px-[10px] lg:px-[50px] py-[10px] h-full w-full">
      <div className="flex flex-col h-full justify-center items-start space-y-[30px]">
        <Logo width={35} />
        {children}
        <div className="text-body-xs">
          By continuing, you consent to our{" "}
          <a className="text-action cursor-pointer">cookie policy</a>,{" "}
          <a className="text-action cursor-pointer">terms of service</a> and{" "}
          <a className="text-action cursor-pointer">privacy policy</a>
        </div>
      </div>
    </div>
  </div>
)

export default AuthLayout
