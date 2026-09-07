# Exigences de sécurité du laboratoire

Ce document décrit les contrôles attendus pour le laboratoire. Une exigence
n'est considérée satisfaite que lorsqu'une configuration et une vérification
techniques la démontrent.

## Réseau

- Ollama est accessible depuis le Mac via `127.0.0.1` seulement.
- L'interface est servie par l'API locale à `127.0.0.1:3210`. Le navigateur
  parle seulement à cette API ; l'API appelle ensuite Ollama à
  `127.0.0.1:11434` et, lors de la future recherche sémantique, Qdrant à
  `127.0.0.1:6333`.
- Aucun fournisseur cloud, recherche web, import documentaire, agent ou MCP
  n'est implémenté. Qdrant local est préparé, vide et lié seulement à
  `127.0.0.1:6333`; le modèle d'embeddings local `embeddinggemma` est installé,
  mais l'API ne les appelle pas encore. La génération augmentée locale utilise
  seulement des extraits lexicaux ACL autorisés.
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
- Les tests démontrent qu'une source refusée n'est ni lue par le lecteur, ni
  récupérée lexicalement, ni transmise dans le contexte RAG du faux Ollama.
  Les mêmes preuves devront couvrir la route sémantique avant son activation.
- Le lecteur documentaire n'accepte aucun chemin client : il résout uniquement
  le chemin déclaré dans la politique, confiné à `demo-documents/`, après ACL.
- La génération augmentée ignore tout champ client `context` ou `sources` : son
  prompt est construit côté serveur depuis la récupération ACL autorisée.
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
- Le composant `audit/security_log.py` ne peut écrire que des événements de
  décision d'accès : horodatage UTC, route, résultat, identité fictive et
  identifiant de ressource lorsqu'ils sont disponibles. Il n'accepte aucun
  prompt, réponse, cookie, JWT ou contenu documentaire.
- Sa destination locale prévue est `.local/audit/access-decisions.jsonl`, hors
  de Git, avec permissions de fichier `600`, répertoire `700`, plafond de 1 Mo
  et un seul fichier de sauvegarde après rotation. La taille maximale est donc
  d'environ 2 Mo.
- La lecture directe `GET /api/documents/<id>`, la vérification
  `GET /api/access-check` et la recherche `POST /api/retrieve` alimentent ce
  journal pour les décisions autorisées et refusées. La recherche ne journalise
  ni ses termes ni ses extraits. Un échec de stockage retourne `503`, afin de
  ne pas exposer de document, décision ACL ou résultat de recherche sans trace.
- Le chat simple `POST /api/chat` journalise aussi la décision d'appeler Ollama,
  sans jamais journaliser son message ou la réponse. Si le journal échoue, le
  message n'est pas envoyé à Ollama.
- Le chat RAG `POST /api/rag-chat` est journalisé avant toute récupération ou
  appel à Ollama, sans question, source, extrait ou réponse. Si le journal
  échoue, aucun document n'est lu et Ollama n'est pas contacté.
- Les journaux seront examinés sans y inscrire de données ou secrets réels.
