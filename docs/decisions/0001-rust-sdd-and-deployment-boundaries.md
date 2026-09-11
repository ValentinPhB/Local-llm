# ADR-0001 — Rust, SDD et frontières de déploiement

Statut : accepté et mis en œuvre ; bascule locale Rust approuvée.

## Décision

Un seul dépôt GitHub ChatPurp et un workspace Cargo : core, contracts, API, web.
API modulaire Axum/Hyper ; frontend Dioxus/WASM servi à la même origine.
Indexeur administratif dans un exécutable distinct, sans route d’écriture
accessible au navigateur. Ollama, Qdrant et les modèles ne sont pas réécrits.

SDD définit exigences puis preuves ; les frontières métier séparent identité,
autorisation, connaissance documentaire, conversation et audit.
Une crate est une unité de code, pas une isolation OS ou un microservice.
Créer plusieurs dépôts maintenant augmenterait les déploiements et contrats
réseau à maintenir sans besoin de scalabilité indépendante démontré.

## Choix des outils

Dioxus est le frontend retenu après qualification sur Mac. Leptos a été écarté
dans cette évaluation à cause d’avis de maintenance dans le graphe étudié :
ce n’est pas une affirmation générale de vulnérabilité ou de supériorité.
Puppeteer Core avec node:test pilote le navigateur qualifié. La CLI
wasm-bindgen complète n’est pas nécessaire ; le moteur cli-support est utilisé
par un petit outil Rust contrôlé.

Les versions et installations ont une seule référence dans
[release management](../release-management.md). Les sondes non retenues ne sont
pas conservées dans le dépôt. Les tests utiles vérifient l’application réelle.

## Conséquences

Le serveur reste souverain ; aucun partage de type avec le frontend ne confère
un droit. Le LLM n’est jamais un mécanisme d’autorisation.
MCP exclus, annuaire fictif et jeu documentaire inchangés.
Le port Rust 3211, son cookie et son audit sont propres à l'application retenue.
Les [limites de sécurité](../security-requirements.md) et la
[méthode](../../specs/README.md) restent applicables.
