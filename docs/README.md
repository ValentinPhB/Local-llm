# ChatPurp — documentation

L’application Rust remplace la version Python, sur le port local 3211.
Les guides décrivent la version Rust ; l’état des processus et les résultats de la séance restent
dans STATUS.md, local et ignoré.

| Guide | Contenu |
| --- | --- |
| [Architecture générale](system-architecture.md) | Composants, langages, ports et dépendances. |
| [Architecture de l’API](local-api-architecture.md) | Modules Rust, responsabilités, contrats HTTP et erreurs. |
| [Flux des requêtes](api-request-flow.md) | Ordre précis des contrôles et circulation des données. |
| [Sécurité et droits](security-requirements.md) | Sessions fictives, matrice ACL, lecture, audit et limites. |
| [Catalogue documentaire](demo-document-catalog.md) | Les quinze fichiers et leurs classifications. |
| [Recherche et indexation](semantic-rag-design.md) | RAG lexical effectif ; adaptateurs sémantiques préparés. |
| [Installation et release management](release-management.md) | Installation de chaque brique, CI, tests, livraison et retour arrière. |
| [Preuves automatisées](security-test-strategy.md) | Exigences reliées aux tests et limites des preuves. |
| [Décision d’architecture](decisions/0001-rust-sdd-and-deployment-boundaries.md) | Dépôt unique, API modulaire, Dioxus et indexeur séparé. |
| [Décision SpecDD](decisions/0002-specdd-contract-pilot.md) | Format .sdd, validation et portée du pilote. |
| [Contrats SDD](../specs/README.md) | SPEC-001 et SPEC-002 ; méthode d’évolution. |
| [Outillage navigateur](../tools/browser-tests/README.md) | Versions, contrôle des binaires et tests réels. |
| [Export OneNote](../tools/export-onenote.mjs) | Génère un HTML autonome et deux PNG à partir des guides courants, dans .local. |

Pas de README racine à la demande de l’utilisateur. Aucun dossier d’archives
n’est nécessaire : les décisions expliquent les choix retenus, Git conserve
les versions publiées précédentes.
