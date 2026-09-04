# LLM Security Lab

Laboratoire local, pédagogique et progressif pour apprendre l'administration,
l'architecture et la sécurité d'une plateforme LLM sur macOS Apple Silicon.

## Objectif

Construire et analyser une architecture locale composée d'Ollama, d'Open WebUI
et, plus tard, de bases documentaires RAG soumises à des contrôles d'accès.

## Principes de sécurité

- Les services restent liés à `localhost` tant qu'une exposition explicite n'est pas décidée.
- Aucun secret, document réel, token ou mot de passe ne doit être ajouté au laboratoire.
- Les données de démonstration sont fictives et explicitement sélectionnées.
- Le LLM ne prend jamais de décision d'autorisation : RBAC et ACL filtrent les ressources avant l'envoi de contexte au modèle.
- Le modèle ne reçoit pas d'accès direct au système de fichiers du Mac.

## État du laboratoire

Initialisation du dépôt et des garde-fous Git en cours. Aucun service, modèle,
conteneur ou port réseau n'est actuellement configuré.
