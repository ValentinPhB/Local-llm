# LLM Security Lab

Laboratoire local, pédagogique et progressif pour apprendre l'administration,
l'architecture et la sécurité d'une plateforme LLM sur macOS Apple Silicon.

## Objectif

Construire et analyser une architecture locale composée d'Ollama, d'une
interface minimale fournie par ce dépôt
et d'une récupération RAG lexicale de documents déjà soumis à des contrôles
d'accès. Une évolution sémantique locale est conçue progressivement.

## Principes de sécurité

- Les services restent liés à `localhost` tant qu'une exposition explicite n'est pas décidée.
- Aucun secret, document réel, token ou mot de passe ne doit être ajouté au laboratoire.
- Les données de démonstration sont fictives et explicitement sélectionnées.
- Le LLM ne prend jamais de décision d'autorisation : RBAC et ACL filtrent les ressources avant l'envoi de contexte au modèle.
- Le modèle ne reçoit pas d'accès direct au système de fichiers du Mac.

## État du laboratoire

Ollama et le modèle local `qwen3:4b` sont disponibles. L'interface minimale
locale est accessible durant son exécution sur `http://127.0.0.1:3210`.

Pour reprendre le projet après une interruption, consulter le fichier local
`STATUS.md`. Il n'est volontairement pas versionné : après un clonage, créer
ce fichier depuis [`STATUS.example.md`](STATUS.example.md), puis y conserver
l'avancement propre à cette machine.

Le sommaire de la documentation est dans [`docs/README.md`](docs/README.md).
Le rôle des deux API locales est décrit dans
[`docs/local-api-architecture.md`](docs/local-api-architecture.md).

La simulation locale d'un annuaire et d'un SSO est expliquée dans
[`docs/demo-sso-authentication.md`](docs/demo-sso-authentication.md). Elle ne représente pas une connexion
à un annuaire d'entreprise.

Les quinze documents fictifs, leurs classifications et leurs limites actuelles
sont décrits dans [`docs/demo-document-catalog.md`](docs/demo-document-catalog.md).

Leur lecture locale autorisée et contrôlée est décrite dans
[`docs/controlled-document-access.md`](docs/controlled-document-access.md).

La récupération lexicale filtrée active est expliquée dans
[`docs/lexical-rag-retrieval.md`](docs/lexical-rag-retrieval.md).

La génération augmentée active est définie dans
[`docs/rag-generation-flow.md`](docs/rag-generation-flow.md).

L’évolution sémantique est décrite dans
[`docs/semantic-rag-design.md`](docs/semantic-rag-design.md). Ses composants
Python (clients et indexeur contrôlé) existent et sont testés avec des faux
services ; elle n’est pas encore activée dans l’API et Qdrant ne contient aucun
document.

Le flux de requête et le détail des composants API sont dans
[`docs/api-request-flow.md`](docs/api-request-flow.md) et
[`docs/local-api-architecture.md`](docs/local-api-architecture.md).

Le contrat de lecture seule prévu pour les futurs MCP est dans
[`docs/mcp-authorization-contract.md`](docs/mcp-authorization-contract.md) ; aucun MCP n'est
encore connecté.

La règle de gestion des versions et mises à jour est décrite dans
[`docs/release-management.md`](docs/release-management.md).
