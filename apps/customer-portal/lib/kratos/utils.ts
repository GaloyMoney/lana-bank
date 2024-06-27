import { LoginFlow, RegistrationFlow, UiNode, UiNodeAttributes } from "@ory/client"

export const kratosUiMessageIds = {
  USER_NOT_EXIST: 4000035,
  OTP_EMAIL_SENT_SIGN_IN: 1010014,
  OTP_EMAIL_SENT_REGISTER: 1040005,
} as const

export const getCsrfToken = (flow: LoginFlow | RegistrationFlow): string | undefined => {
  for (const node of flow.ui.nodes) {
    if (isInputNode(node)) {
      if (node.attributes.name === "csrf_token") {
        return node.attributes.value
      }
    }
  }
}

export function isInputNode(
  node: UiNode,
): node is UiNode & { attributes: UiNodeAttributes & { name: string; value?: string } } {
  return (
    "attributes" in node &&
    typeof node.attributes === "object" &&
    "name" in node.attributes
  )
}

export const emailParserFromUiNodeRegister = (nodes: UiNode[]): string | null => {
  let email = null
  nodes.forEach((node) => {
    const attributes = node.attributes as { name?: string; value?: string }
    if (attributes.name === "traits.email") {
      email = attributes.value
    }
  })
  return email
}

export const emailParserFromUiNodeLogin = (nodes: UiNode[]): string | null => {
  let email = null
  nodes.forEach((node) => {
    const attributes = node.attributes as { name?: string; value?: string }
    if (attributes.name === "identifier") {
      email = attributes.value
    }
  })
  return email
}
