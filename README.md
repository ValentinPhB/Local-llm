# LLM Security Lab

Laboratoire local, pédagogique et progressif pour apprendre l'administration,
l'architecture et la sécurité d'une plateforme LLM sur macOS Apple Silicon.

## Objectif

Construire et analyser une architecture locale composée d'Ollama, d'une
interface minimale fournie par ce dépôt
et, plus tard, d'une récupération RAG de documents déjà soumis à des contrôles
d'accès.

## Principes de sécurité

- Les services restent liés à `localhost` tant qu'une exposition explicite n'est pas décidée.
- Aucun secret, document réel, token ou mot de passe ne doit être ajouté au laboratoire.
- Les données de démonstration sont fictives et explicitement sélectionnées.
- Le LLM ne prend jamais de décision d'autorisation : RBAC et ACL filtrent les ressources avant l'envoi de contexte au modèle.
- Le modèle ne reçoit pas d'accès direct au système de fichiers du Mac.

## État du laboratoire

Ollama et le modèle local `qwen3:4b` sont disponibles. L'interface minimale
locale est accessible durant son exécution sur `http://127.0.0.1:3210`.

Pour reprendre le projet après une interruption, consulter d'abord
[`STATUS.md`](STATUS.md).

Le rôle des deux API locales est décrit dans
[`docs/local-api-architecture.md`](docs/local-api-architecture.md).

La simulation locale d'un annuaire et d'un SSO est expliquée dans
[`docs/demo-sso.md`](docs/demo-sso.md). Elle ne représente pas une connexion
à un annuaire d'entreprise.

Les quinze documents fictifs, leurs classifications et leurs limites actuelles
sont décrits dans [`docs/demo-documents.md`](docs/demo-documents.md).

Leur lecture locale autorisée, sans RAG ni envoi au modèle, est décrite dans
[`docs/controlled-document-reader.md`](docs/controlled-document-reader.md).

La conception de la récupération filtrée qui précédera le RAG est dans
[`docs/controlled-retrieval-design.md`](docs/controlled-retrieval-design.md).

Le contrat de lecture seule prévu pour les futurs MCP est dans
[`docs/mcp-authorization.md`](docs/mcp-authorization.md) ; aucun MCP n'est
encore connecté.

La règle de gestion des versions et mises à jour est décrite dans
[`docs/release-management.md`](docs/release-management.md).
