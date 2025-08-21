const translations: Record<string, Record<string, string>> = {
  en: {
    duplicate_username: 'Username already taken',
    duplicate_email: 'Email already taken',
    invalid_credentials: 'Invalid credentials',
    missing_token: 'Authentication required',
    invalid_token: 'Invalid token',
    config_error: 'Server misconfiguration',
    server_error: 'Server error'
  },
  fr: {
    duplicate_username: "Nom d'utilisateur déjà pris",
    duplicate_email: "E-mail déjà utilisé",
    invalid_credentials: 'Identifiants invalides',
    missing_token: 'Authentification requise',
    invalid_token: 'Jeton invalide',
    config_error: 'Erreur de configuration serveur',
    server_error: 'Erreur serveur'
  }
}

let locale = (import.meta.env.VITE_I18N_LOCALE as string) || 'fr'

export function setLocale(l: string) { locale = l }
export function t(key: string) {
  return translations[locale] && translations[locale][key] ? translations[locale][key] : key
}
