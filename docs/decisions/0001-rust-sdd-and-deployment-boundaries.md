# ADR-0001 — Rust, SDD et frontières de déploiement

Date : 2026-09-11. Statut : direction acceptée par l'utilisateur.
Mise en œuvre : non commencée ; l'application active est en Python.

## Contexte

Le laboratoire local possède déjà un moteur ACL indépendant, un lecteur
contrôlé, une couche sémantique à dépendances injectées et une CI Python.
L'orchestration HTTP et les sessions sont concentrées dans `ui/server.py`.
L'utilisateur veut apprendre en migrant vers Rust et en travaillant à partir
de spécifications, tout en conservant la sécurité et des tests automatisés.

## Décisions

1. Conserver le dépôt GitHub existant. Un workspace Cargo hébergera plusieurs
   composants ; aucun nouveau dépôt ni ensemble de microservices n'est requis.
2. Utiliser le SDD pour livrer par petites fonctionnalités, avec des limites
   métier explicites. Ne pas confondre dossiers techniques et domaines.
3. Migrer le code applicatif écrit pour le lab : API, indexeur et interface.
   Ollama, Qdrant et les modèles restent des dépendances externes ; ils ne sont
   pas réécrits pour imposer un langage unique à toute la chaîne.
4. Viser une API Rust modulaire avec Axum et une interface Leptos en rendu
   navigateur (WebAssembly), servie à la même origine que l'API. HTML/CSS et
   JavaScript généré de chargement restent nécessaires. Le rendu serveur et
   l'hydratation ne sont pas requis pour cette première cible.
5. Viser un indexeur administratif distinct, lancé à la demande. L'API
   utilisateur n'expose pas les opérations d'écriture de l'index.
6. Conserver les règles de sécurité de [AGENTS.md](../../AGENTS.md) et migrer
   progressivement, à partir d'une session fictive et d'une lecture contrôlée.
   Les tests Python sont une référence à confronter aux specs, pas une preuve
   suffisante de conformité de la future version Rust.

Les versions, noms et nombre exact des crates seront décidés dans les plans.
Le choix du framework ne permet jamais de déplacer une décision d'autorisation
vers le navigateur, y compris si le client et le serveur partagent des types.

## Responsabilités proposées

| Ensemble | Responsabilité et limite |
| --- | --- |
| Identité et autorisation | Vérifier les assertions fictives et décider les droits ; ne lit pas les documents |
| Connaissance documentaire | Catalogue, lecture, passages, indexation et recherche ; ne crée pas de droit |
| Conversation | Construire le contexte autorisé et appeler Ollama ; ne prend pas de décision ACL |
| Audit | Écrire des décisions minimales ; ne reçoit pas le contenu documentaire ou conversationnel |
| Outils, futur | Contrôler chaque action MCP déclarée ; aucun connecteur implicite |

Ces ensembles orientent les modules. Ce ne sont ni cinq microservices à créer,
ni un engagement à utiliser une crate par ligne.

## Cible d'exécution

```text
Navigateur : interface Rust compilée en WASM
    -> API Rust locale : session -> ACL -> audit -> lecture/recherche
        -> documents autorisés
        -> Qdrant : recherche filtrée (étape ultérieure)
        -> Ollama : génération et embeddings selon le cas d'usage

Indexeur administratif Rust, à la demande (étape ultérieure)
    -> documents explicitement sélectionnés
    -> Ollama : embeddings
    -> Qdrant : écriture contrôlée
```

L'indexeur et l'API pourront partager du code métier. Séparer deux exécutables
ne sépare pas leurs droits : les permissions effectives sur les fichiers et
Qdrant devront être spécifiées et testées avant activation de l'indexation.
Cette décision n'introduit donc aucune garantie d'isolement système déjà active.

## Conséquences et alternatives

- Le dépôt unique permet de modifier ensemble contrats, code et tests.
  Cargo ne garantit pas à lui seul le bon sens des dépendances : le plan puis
  les contrôles automatisés devront rendre ces frontières vérifiables.
- La compilation Rust/WASM ajoute des outils et un coût de build à mesurer
  sur le Mac M1. Aucun gain de vitesse d'inférence n'est promis.
- Des services et dépôts distincts restent possibles si une frontière de
  privilèges, une livraison ou une exploitation indépendante les justifie.
- Une réécriture complète d'un coup ferait perdre nos points de comparaison.
  Chaque tranche doit avoir des critères de bascule et de retour arrière.
- La migration de la lecture précède le branchement du RAG sémantique. La
  couche sémantique Python préparée reste conservée, sans nouvelle activation.

## Références

- [Méthode et index des specs](../../specs/README.md)
- [Architecture actuellement exécutée](../system-architecture.md)
- [Cargo workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [Modes de rendu Leptos](https://book.leptos.dev/getting_started/index.html)
