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

- Expliquer l'objectif, l'impact, les risques et la vérification avant chaque changement significatif.
- Limiter chaque étape à un changement cohérent et vérifier son résultat.
- Préserver les changements existants de l'utilisateur et ne pas effectuer d'action destructive sans autorisation explicite.
- Préférer le moindre privilège, des identifiants dédiés et une journalisation adaptée.

## MCP et outils futurs

- Ne pas ajouter, configurer ou connecter de serveur MCP sans décision explicite.
- Définir pour chaque outil les permissions minimales, les actions interdites et les mécanismes d'audit avant son utilisation.
