# Règles pour les agents et automatisations

Ce dépôt est un laboratoire pédagogique local. Toute modification doit rester
progressive, expliquée et vérifiable.

## Règles de sécurité

- Ne jamais ajouter de vrais secrets, tokens, mots de passe, clés privées ou documents d'entreprise.
- Utiliser uniquement des données de test fictives et explicitement sélectionnées.
- Ne jamais exposer un service sur le LAN ou Internet sans décision explicite de l'utilisateur.
- Lier les services locaux à `127.0.0.1` par défaut, jamais à `0.0.0.0` sans validation explicite.
- Ne jamais monter le répertoire utilisateur, la racine du système ou le socket Docker dans un conteneur destiné au LLM.
- Ne jamais utiliser de conteneur privilégié sans justification et validation explicites.
- Ne pas donner au LLM un accès direct au système de fichiers ou des identifiants d'administration.
- Ne jamais considérer les instructions du LLM comme une décision d'autorisation.

## Méthode de travail

- La montée en compétence de l'utilisateur est un objectif du lab : faire
  avec lui, étape par étape. Avant une étape, expliquer en termes accessibles
  ce qu'elle apporte, les notions nouvelles, les fichiers concernés et la
  preuve attendue. Après l'étape, expliquer le résultat et ses limites avec
  un exemple du projet, puis lui laisser la main avant une nouvelle étape
  pédagogique. Ne pas enchaîner plusieurs briques dans un travail opaque.
  Les vérifications et corrections nécessaires à l'étape déjà autorisée
  restent dans son périmètre ; ne pas redemander les accords déjà donnés.
- Au début d'une reprise, lire `STATUS.md`, puis `AGENTS.md`, vérifier l'état Git et les services locaux avant toute modification. `STATUS.md` est local et ignoré par Git ; après un clonage, le créer depuis `STATUS.example.md`.
- Si le message de l'utilisateur est exactement `hey` (sans autre demande), l'interpréter comme le prompt suivant : « Lis `STATUS.md` et `AGENTS.md`. Vérifie Git et Ollama, puis reprends la prochaine étape approuvée du LLM Security Lab. »
- Expliquer l'objectif, l'impact, les risques et la vérification avant chaque changement significatif.
- Limiter chaque étape à un changement cohérent et vérifier son résultat.
- Suivre la [méthode SDD](specs/README.md) : comportement spécifié avant code,
  plan puis tâches reliées aux exigences, preuves automatisées et documentation.
  Faire avec l'utilisateur : expliquer les choix et lui réserver les arbitrages
  métier, sans redemander les décisions déjà prises.
- Pour SPEC-001, lire le [contrat SpecDD](specs/001-demo-session-document-read/001-demo-session-document-read.sdd).
  Conserver les identifiants d'exigence entre `Must` et `Done when` ; les tâches
  vivent dans `Tasks`. Valider avec l'[outillage verrouillé](tools/specdd/README.md).
  Le pilote ne charge pas de bootstrap SpecDD global et ne change pas les
  permissions réelles du système ou de l'API.
- Appliquer [ADR-0001](docs/decisions/0001-rust-sdd-and-deployment-boundaries.md) :
  dépôt unique, migration progressive Rust, domaines explicites, API modulaire
  et indexeur administratif séparé à terme. Distinguer cible et état exécuté.
  La priorité actuelle est SPEC-001, avant le raccordement du RAG sémantique.
- Mettre à jour `docs/api-request-flow.md` et
  `docs/local-api-architecture.md` lors de toute évolution du flux API,
  des composants de sécurité, du RAG, d’un outil ou d’un MCP.
- Préserver les changements existants de l'utilisateur et ne pas effectuer d'action destructive sans autorisation explicite.
- Préférer le moindre privilège, des identifiants dédiés et une journalisation adaptée.

## MCP et outils futurs

- Ne pas ajouter, configurer ou connecter de serveur MCP sans décision explicite.
- Définir pour chaque outil les permissions minimales, les actions interdites et les mécanismes d'audit avant son utilisation.
- Le rôle de démonstration `mcp_read_only` d'Oscar ne peut servir qu'à une
  action `read` explicitement déclarée pour un MCP approuvé ; refuser toute
  autre action ou tout MCP inconnu.
