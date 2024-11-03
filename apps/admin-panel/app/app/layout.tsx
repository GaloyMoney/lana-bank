import NavBar from "./navbar"

const AppLayout: React.FC<React.PropsWithChildren> = ({ children }) => {
  return (
    <div className="bg-soft h-screen w-screen flex">
      <NavBar />
      <div className="bg-black w-full h-full">{children}</div>
    </div>
  )
}

export default AppLayout
