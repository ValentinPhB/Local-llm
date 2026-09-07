# Exigences de sécurité

Ce document décrit les contrôles attendus pour le laboratoire. Une exigence
n'est considérée satisfaite que lorsqu'une configuration et une vérification
techniques la démontrent.

## Réseau

- Ollama est accessible depuis le Mac via `127.0.0.1` seulement.
- L'interface locale est liée à `127.0.0.1:3210` et sa seule cible est `http://127.0.0.1:11434`.
- Aucun fournisseur cloud, recherche web, import documentaire, embeddings,
  base vectorielle, génération augmentée, agent ou MCP n'est implémenté. La
  récupération lexicale locale retourne seulement des extraits ACL autorisés.
- Aucun port n'est publié sur le LAN ou Internet sans décision explicite et documentée.
- L'API Ollama ne doit pas être exposée directement à des utilisateurs non authentifiés.
- Les fonctions cloud et la recherche web d'Ollama sont désactivées pendant la phase locale du laboratoire.

## Données et secrets

- Les scénarios de test utilisent exclusivement des documents et secrets fictifs.
- Les quinze documents de démonstration restent sous `demo-documents/`, sont
  versionnés et classifiés ; aucun document réel ne les remplace sans décision
  explicite et revue des contrôles.
- Les fichiers `.env`, clés privées, credentials et tokens réels restent hors de Git.
- La clé de signature du SSO fictif est éphémère et n'est jamais écrite dans le
  dépôt, un fichier de configuration ou les journaux.
- Les données persistantes des services sont identifiées avant leur création.
- Aucun répertoire personnel complet ou racine du système n'est monté dans un conteneur.

## Autorisation et RAG

- La simulation locale vérifie signature, émetteur, audience et expiration du
  jeton avant d'utiliser son sujet ou ses groupes. Elle ne constitue pas une
  authentification réelle, car l'identité de démonstration est choisie librement.
- Une intégration entreprise vérifiera les jetons de l'IdP réel avant toute
  recherche documentaire ; l'API ne recevra pas le mot de passe utilisateur.
- RBAC et ACL décident de l'accès avant l'envoi de passages au modèle.
- Le modèle ne décide jamais si un document est accessible.
- Une instruction contenue dans un prompt ou un document ne change jamais les permissions.
- Les tests du lecteur démontrent déjà qu'une source refusée n'est pas lue.
  Avant d'ajouter le RAG, ils devront aussi démontrer qu'elle n'est ni
  récupérée ni transmise dans le contexte du modèle.
- Le lecteur documentaire n'accepte aucun chemin client : il résout uniquement
  le chemin déclaré dans la politique, confiné à `demo-documents/`, après ACL.
- Les traces de raisonnement éventuelles sont traitées comme des données potentiellement sensibles : elles ne doivent pas être affichées ou journalisées par défaut.
- Une option client telle que `think: false` n'est pas considérée comme une garantie de suppression de ces traces sans test du contenu réellement reçu.
- Les routes de chat ne doivent jamais accepter un identifiant, des groupes ou
  des rôles choisis par le navigateur. Elles utilisent la session vérifiée.
- Le rôle futur `mcp_read_only` n'autorise que des actions `read` déclarées
  d'un MCP approuvé ; il ne vaut jamais pour une action inconnue, d'écriture,
  d'exécution, d'administration ou de suppression.

## Conteneurs et outils futurs

- Les conteneurs reçoivent le minimum de privilèges et de capacités nécessaire.
- Aucun conteneur destiné au LLM n'accède au socket Docker.
- Tout MCP ou outil externe nécessite des identifiants dédiés, des permissions minimales et des tests de refus.

## Traçabilité

- Les modifications de configuration sont versionnées dans Git.
- Les commandes d'installation et leurs vérifications sont documentées.
- Les journaux sont examinés sans y inscrire de données ou secrets réels.
