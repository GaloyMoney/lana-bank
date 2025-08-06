import { KeycloakAuthForm } from "@/components/auth/keycloak-form"
import { AuthTemplateCard } from "@/components/auth/auth-template-card"

function Auth() {
  return (
    <AuthTemplateCard>
      <KeycloakAuthForm />
    </AuthTemplateCard>
  )
}

export default Auth
