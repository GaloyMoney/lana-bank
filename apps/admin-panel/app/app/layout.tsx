import NavBar from "./navbar"
import { SearchInput } from "./search"
import CreateButton from "./create"

import ApolloServerWrapper from "@/lib/apollo-client/server-wrapper"

const AppLayout: React.FC<React.PropsWithChildren> = ({ children }) => {
  return (
    <ApolloServerWrapper>
      <div className="bg-soft h-full w-full flex flex-col md:flex-row">
        <NavBar />
        <div className="flex-1 pt-[72px] md:pt-2 p-2 max-h-screen overflow-hidden">
          <div className="p-2 border rounded-md flex flex-col w-full h-full">
            <div className="md:flex gap-2 hidden pb-2">
              <SearchInput />
              <CreateButton />
            </div>
            <main className="h-full overflow-y-auto no-scrollbar">{children}</main>
          </div>
        </div>
      </div>
    </ApolloServerWrapper>
  )
}

export default AppLayout
