Backoffice minimal pour EcoBlock API Kernel

Prérequis
- Node.js 18+ et npm/yarn

Démarrage local

1. Se placer dans le répertoire:

   cd web-admin

2. Installer les dépendances:

   npm install

3. Démarrer en mode développement:

   npm run dev

Le front démarre par défaut sur http://localhost:5173.

Configuration API

Le front utilise la variable d'environnement `VITE_API_BASE` pour appeler l'API. Par défaut elle pointe sur `http://localhost:3000`.

Exemples:

  VITE_API_BASE=http://localhost:3000 npm run dev

Notes
- L'interface est minimale: affiche la liste des blocks via GET /tangle/blocks (adapter l'endpoint si nécessaire).
- Je peux ajouter des pages pour utilisateurs, modération, et formulaires CRUD si vous le souhaitez.
