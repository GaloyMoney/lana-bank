import { Button, Input } from "@/components"

const Login: React.FC = () => (
  <>
    <h1 className="text-heading-h3">Sign In</h1>
    <div className="space-y-[10px]">
      <div className="text-title-md">Welcome to Lana Bank Admin Panel</div>
      <div className="text-body-md">Enter your email address to continue</div>
    </div>
    <div className="space-y-[20px] w-full">
      <Input
        label="Your email"
        type="email"
        autofocus
        placeholder="Please enter your email address"
      />
      <Button title="Submit" />
    </div>
  </>
)

export default Login
