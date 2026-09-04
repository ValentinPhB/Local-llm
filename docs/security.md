# Exigences de sécurité

Ce document décrit les contrôles attendus pour le laboratoire. Une exigence
n'est considérée satisfaite que lorsqu'une configuration et une vérification
techniques la démontrent.

## Réseau

- Ollama est accessible depuis le Mac via `127.0.0.1` seulement.
- L'interface locale est liée à `127.0.0.1:3210` et sa seule cible est `http://127.0.0.1:11434`.
- Aucun fournisseur cloud, recherche web, import documentaire, agent ou MCP n'est implémenté durant cette phase.
- Aucun port n'est publié sur le LAN ou Internet sans décision explicite et documentée.
- L'API Ollama ne doit pas être exposée directement à des utilisateurs non authentifiés.
- Les fonctions cloud et la recherche web d'Ollama sont désactivées pendant la phase locale du laboratoire.

## Données et secrets

- Les scénarios de test utilisent exclusivement des documents et secrets fictifs.
- Les fichiers `.env`, clés privées, credentials et tokens réels restent hors de Git.
- Les données persistantes des services sont identifiées avant leur création.
- Aucun répertoire personnel complet ou racine du système n'est monté dans un conteneur.

## Autorisation et RAG

- L'authentification identifie l'utilisateur avant toute recherche documentaire.
- RBAC et ACL décident de l'accès avant l'envoi de passages au modèle.
- Le modèle ne décide jamais si un document est accessible.
- Une instruction contenue dans un prompt ou un document ne change jamais les permissions.
- Les tests démontrent qu'une source non autorisée n'est pas récupérée dans le contexte du modèle.
- Les traces de raisonnement éventuelles sont traitées comme des données potentiellement sensibles : elles ne doivent pas être affichées ou journalisées par défaut.
- Une option client telle que `think: false` n'est pas considérée comme une garantie de suppression de ces traces sans test du contenu réellement reçu.

## Conteneurs et outils futurs

- Les conteneurs reçoivent le minimum de privilèges et de capacités nécessaire.
- Aucun conteneur destiné au LLM n'accède au socket Docker.
- Tout MCP ou outil externe nécessite des identifiants dédiés, des permissions minimales et des tests de refus.

## Traçabilité

- Les modifications de configuration sont versionnées dans Git.
- Les commandes d'installation et leurs vérifications sont documentées.
- Les journaux sont examinés sans y inscrire de données ou secrets réels.
