# Architecture générale de ChatPurp

## Périmètre

Laboratoire local sur Mac Apple Silicon, 16 Gio de RAM. Le code applicatif
propre au lab est en Rust : interface, règles métier, API et indexeur.
Ollama, ses modèles, Qdrant et le navigateur restent des logiciels externes.
« Full Rust » ne signifie pas réécrire ces logiciels : HTML/CSS et le
JavaScript de liaison de WebAssembly restent nécessaires.

La version Rust remplace le service Python après bascule approuvée et tests.
Le port applicatif est 3211 ; aucun service du projet n'utilise désormais 3210.

## Composants et ports

| Composant | Technologie | Adresse / rôle |
| --- | --- | --- |
| Interface | Dioxus 0.7.10 → WebAssembly | Servie par la même origine que l’API, sans serveur frontend séparé. |
| API | Rust 1.98.1, Axum 0.8.9, Hyper 1.11.1, Tokio 1.53.1 | Écoute fixe 127.0.0.1:3211 ; sessions, ACL, lecture, recherche, chat. |
| Génération | Ollama 0.33.3, qwen3:4b | 127.0.0.1:11434 ; seul le serveur l’appelle. |
| Embeddings préparés | Ollama, embeddinggemma | Même serveur, endpoint /api/embed ; hors routes actives. |
| Base vectorielle préparée | Qdrant v1.19.1-unprivileged, image par digest | 127.0.0.1:6333 ; aucune indexation du lab exécutée pendant la migration. |
| Indexeur administratif | Binaire Rust chatpurp-index | Aucun port ; plan hors réseau ou remplacement explicitement demandé. |
| Tests navigateur | Node 22.23.2, Puppeteer Core 25.10.0, Chromium 153.0.8010.36 | Outils de développement dédiés, pas des services applicatifs. |

## Dépendances internes

~~~text
chatpurp-web ──→ chatpurp-contracts
chatpurp-api ──→ chatpurp-core
             └→ chatpurp-contracts
~~~

Le core porte les règles et des interfaces de dépendances injectées. Il ne
dépend ni de HTTP, ni d’Ollama, ni de rustix. L’API implémente ces interfaces.
Le frontend ne reçoit ni la politique complète ni une clé de signature.
Un test vérifie le graphe réel de compilation WASM : aucun paquet serveur
JWT, filesystem natif ou API n’y est joignable.

L’API et l’indexeur partagent la crate native mais constituent deux exécutables.
Cette séparation évite une route d’indexation exposée au navigateur ; elle ne
sépare pas les permissions du compte macOS. Plusieurs dépôts ou microservices
n’apporteraient pas automatiquement davantage de sécurité.

## Ressources locales

Compilation séquentielle (CARGO_BUILD_JOBS=1). L’API accepte au plus seize
connexions simultanées et un seul appel de génération à la fois, sans file
illimitée de prompts. qwen3:4b occupe environ 2,5 Go sur disque ; les mesures
antérieures d’inférence étaient d’environ 3,2 Go, pas une limite garantie.
Le modèle d’embeddings occupe environ 621 Mo sur disque.

La toolchain, les builds, navigateurs, caches et audits résident dans .local,
hors Git. Un build n’est pas un modèle ; un cache disque n’est pas une
consommation permanente de RAM. Aucune exposition LAN, aucun cloud, aucun MCP.

Voir le [flux](api-request-flow.md), les [modules](local-api-architecture.md)
et l’[exploitation](release-management.md).
