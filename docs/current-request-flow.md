# Flux d’exécution actuel

Ce document est la référence du trajet d’une requête dans le laboratoire. Il
doit être mis à jour lors de toute modification de session, ACL, lecture,
récupération, génération, outil ou MCP.

## Principe

Le LLM ne vérifie aucun droit et n’ouvre aucun fichier. L’API locale Python est
le gardien : elle vérifie la session, applique l’ACL, puis ne transmet à Ollama
que les données autorisées.

```text
Utilisateur -> UI -> API locale Python -> Ollama -> API locale -> UI
                    |                 
                    -> session / ACL / documents / récupération
```

## 1. Création d’une session de démonstration

```text
1. L'utilisateur choisit Alice, Bob, Charlie ou Oscar dans l'UI.
2. L'UI appelle POST /api/demo-session avec cet identifiant fictif.
3. L'API vérifie l'identifiant dans l'annuaire fictif.
4. L'API émet un JWT HS256 valable 15 minutes.
5. Le navigateur reçoit ce jeton dans un cookie HttpOnly, SameSite=Strict.
```

Le choix est libre : il simule une assertion d’identité, mais n’est pas une
authentification réelle. Dans une entreprise, un IdP émettrait le jeton.

## 2. Décision d’accès commune

Pour toute route protégée, l’API applique ce trajet avant la lecture ou
l’inférence :

```text
Cookie de session
-> signature + expiration + issuer + audience vérifiés
-> groupes du jeton
-> rôles applicatifs
-> ACL de la ressource
-> autorisation ou refus
```

Un refus renvoie une erreur au navigateur. Le modèle ne participe jamais à
cette décision.

## 3. Lecture directe d’un document

Route : `GET /api/documents/<resource_id>`.

```text
1. API vérifie la session et l’ACL du resource_id.
2. Si refus : 403 ; le fichier n’est pas ouvert.
3. Si autorisation : l’API utilise seulement le chemin déclaré dans la politique.
4. Le chemin est confiné à demo-documents/ ; taille et métadonnées sont vérifiées.
5. Le contenu est retourné à l’UI, avec Cache-Control: no-store.
```

Exemple : Oscar peut lire `public-welcome`, mais pas `public-glossary`, RH ou IT.

## 4. Récupération contrôlée

Route : `POST /api/retrieve` avec `{"query":"…"}`.

```text
1. API vérifie session, groupes et rôles.
2. API calcule la liste des resource_id autorisés.
3. Seuls ces fichiers sont lus par le récupérateur.
4. Recherche lexicale locale par mots-clés.
5. Au plus trois extraits de 500 caractères sont retournés à l’UI.
```

Il n’y a pas d’embeddings, de base vectorielle ou d’index persistant. Une
ressource interdite n’est ni lue, ni classée, ni retournée.

## 5. Chat simple

Route : `POST /api/chat` avec `{"message":"…"}`.

```text
1. API vérifie la session.
2. API transmet uniquement le message à Ollama.
3. Ollama produit une réponse.
4. API retire une éventuelle trace Qwen jusqu’à </think>.
5. API retourne la réponse à l’UI.
```

Le chat simple ne lit ni ne transmet aucun document.

## 6. Chat RAG contrôlé

Route : `POST /api/rag-chat` avec `{"message":"…"}`.

```text
1. API vérifie la session et recalcule les droits ACL.
2. API récupère au plus trois extraits parmi les seules ressources autorisées.
3. API construit un message système avec ces extraits.
4. API ajoute le message utilisateur.
5. API appelle Ollama avec ce contexte construit côté serveur.
6. API retourne la réponse et les identifiants des sources utilisées.
```

Le navigateur ne peut pas fournir `context`, `sources`, chemin, rôle ou liste
de documents : ces données sont ignorées. Pour Oscar, le contexte RAG ne peut
contenir qu’un extrait de `public-welcome`.

## 7. Ce que le LLM peut et ne peut pas faire

Le LLM reçoit du texte ; il ne reçoit ni permission, ni chemin local, ni outil,
ni MCP, ni accès au système de fichiers. Il peut écrire une réponse erronée ou
une suggestion, mais il ne peut pas ouvrir lui-même un dossier RH.

Un futur MCP suivra ce même flux : demande du LLM -> passerelle API -> contrôle
identité + action + paramètres -> exécution ou refus.
