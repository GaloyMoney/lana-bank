import { get } from "lodash"

import enTranslations from "../../messages/en.json"
import esTranslations from "../../messages/es.json"

type TranslationsType = typeof enTranslations

interface Translator {
  (path: string, params?: Record<string, string | number>): string
}

const translations: Record<string, TranslationsType> = {
  en: enTranslations,
  es: esTranslations,
}

function createTranslator(): Translator {
  const getTranslation = (
    path: string,
    params?: Record<string, string | number>,
  ): string => {
    const locale = Cypress.env("TEST_LANGUAGE")
    const translationObj = translations[locale]
    let value = get(translationObj, path, path) as string
    if (params) {
      Object.entries(params).forEach(([key, val]) => {
        value = value.replace(new RegExp(`{${key}}`, "g"), String(val))
      })
    }
    return value
  }

  const translator = ((path: string, params?: Record<string, string | number>) => {
    return getTranslation(path, params).toString()
  }) as Translator

  return translator
}

export const t = createTranslator()
