# LLM Security Lab

Laboratoire local, pédagogique et progressif pour apprendre l'administration,
l'architecture et la sécurité d'une plateforme LLM sur macOS Apple Silicon.

## Objectif

Construire et analyser une architecture locale composée d'Ollama, d'une
interface minimale fournie par ce dépôt
et, plus tard, de bases documentaires RAG soumises à des contrôles d'accès.

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
