import Input from "@/components/input"
import Logo from "@/components/logo"

import { BeakerIcon } from "@heroicons/react/24/solid"

export default function Home() {
  return (
    <>
      <Input
        label="Your email"
        placeholder="Name"
        type="text"
        leftNode={<BeakerIcon className="h-8 text-primary" />}
        rightNode={<BeakerIcon className="h-8 text-primary" />}
      />
      <Logo variant="neutral" />
    </>
  )
}
