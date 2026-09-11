# Architecture détaillée de l'API locale

État exécuté : Python. La migration Rust suit
[SPEC-001](../specs/001-demo-session-document-read/001-demo-session-document-read.sdd), encore proposée ;
aucune route n'a été remplacée. La cible est décrite dans
[ADR-0001](decisions/0001-rust-sdd-and-deployment-boundaries.md).

## Rôle

L’API du laboratoire est le processus `python3 ui/server.py`. Elle écoute sur
`127.0.0.1:3210`, sert l’interface et applique les contrôles de sécurité avant
tout accès à un document ou appel à Ollama.

Elle ne modifie pas Ollama : elle utilise son API native locale sur
`127.0.0.1:11434`.

```text
Navigateur -- 127.0.0.1:3210 --> API Python -- 127.0.0.1:11434 --> Ollama
```

Les deux services sont limités à `127.0.0.1` : ils ne sont pas accessibles
depuis le réseau local ou Internet. L’API Ollama existe dès que l’application
Ollama est démarrée ; l’API Python existe seulement pendant l’exécution de
`ui/server.py`.

## Composants

| Fichier | Responsabilité |
| --- | --- |
| `ui/server.py` | Routes HTTP, validation, sessions, orchestration ACL/RAG et relais Ollama. |
| `ui/index.html` | Interface locale ; n’est jamais une source de permission. |
| `identity/demo_sso.py` | Annuaire fictif, émission et vérification du JWT court. |
| `access_control/engine.py` | Traduction groupes -> rôles et décision RBAC/ACL déterministe. |
| `document_store/reader.py` | Lecture confinée au chemin déclaré, après autorisation. |
| `document_store/retriever.py` | Classement lexical des seuls documents déjà autorisés. |
| `semantic_retrieval/clients.py` | Adaptateurs Ollama embeddings, recherche Qdrant et writer Qdrant fixe, testés mais non raccordés à l'API. |
| `semantic_retrieval/indexer.py` | Indexeur contrôlé : relit les seuls documents de politique, découpe et remet un lot à un writer injecté. |
| `audit/security_log.py` | Événements d'audit minimaux et stockage local borné pour ACL, lecture, récupération, chat et RAG. |
| `config/demo-idp/directory.json` | Quatre identités et leurs groupes fictifs. |
| `config/access-control/demo-policy.json` | Groupes, rôles, ACL et chemins des 15 documents. |

## Routes

| Route | Rôle | Accès documentaire / Ollama |
| --- | --- | --- |
| `GET /healthz` | Vérifie que l’API répond. | Ni document ni Ollama. |
| `POST /api/demo-session` | Crée la session fictive signée. | Ni document ni Ollama. |
| `GET /api/session` | Vérifie et décrit la session. | Ni document ni Ollama. |
| `GET /api/access-check` | Retourne une décision ACL journalisée. | Ne lit pas de document. |
| `GET /api/documents/<id>` | Lit un document autorisé. | Document seulement. |
| `POST /api/retrieve` | Retourne des extraits autorisés après audit. | Documents autorisés seulement. |
| `POST /api/chat` | Chat simple après audit. | Ollama, sans document. |
| `POST /api/rag-chat` | Chat avec extraits autorisés après audit. | Documents autorisés puis Ollama. |
| `POST /api/logout` | Supprime le cookie côté navigateur. | Ni document ni Ollama. |

Les routes `POST /api/chat` et `POST /api/rag-chat` acceptent un JSON contenant
seulement `{"message":"…"}`. Un message absent, vide, non textuel ou trop
long est refusé avec `400`. Les champs client supplémentaires comme `context`,
`sources`, rôle ou chemin ne modifient jamais la sécurité ou le contexte RAG.

## Relais vers Ollama

L’API impose le modèle `qwen3:4b`, l’URL locale fixe
`http://127.0.0.1:11434/api/chat`, `stream: false` et `think: false`. Après la
réponse, elle retire tout contenu situé avant `</think>`, car Qwen peut ignorer
la demande `think: false`. Elle retourne `502` si Ollama est indisponible ou si
sa réponse est invalide.

Ces adaptations appartiennent à notre API ; elles ne modifient ni les routes,
ni les modèles, ni la configuration d’Ollama.

## État et données

- La clé JWT est créée aléatoirement au démarrage et reste seulement en mémoire.
- Les sessions expirent après 15 minutes et un redémarrage les invalide.
- Les prompts et réponses ne sont pas journalisés par le serveur.
- Les décisions de lecture directe, de vérification ACL, de recherche, de chat
  simple et de RAG sont écrites dans `.local/audit/access-decisions.jsonl`,
  hors Git, limité à 1 Mo avec une sauvegarde et des permissions privées. Les
  termes de recherche, extraits, messages, sources et réponses ne sont pas
  journalisés.
- Les documents sont des fichiers Markdown fictifs versionnés dans Git.
- Aucun index persistant ni embedding intégré à l'API n’existe. Qdrant local
  est démarré mais sa collection est vide et l'API ne s'y connecte pas encore.
  Le modèle local `embeddinggemma` est installé dans Ollama mais n'est pas
  encore consommé par ce processus. Les adaptateurs sémantiques et l'indexeur
  contrôlé sont isolés, n'ont pas de route HTTP et aucun d'eux n'est encore
  instancié pour écrire dans Qdrant réel. Aucun MCP n'existe.

## Contrôles de frontière

1. Le navigateur ne choisit jamais ses rôles ou les sources RAG.
2. L’API vérifie le jeton avant toute route protégée.
3. L’ACL est évaluée avant la lecture physique d’un fichier.
4. Le lecteur refuse les chemins client et reste dans `demo-documents/`.
5. Le récupérateur reçoit uniquement des identifiants déjà autorisés.
6. Le chat RAG construit son contexte côté serveur, puis retourne les sources.
7. Ollama ne reçoit jamais un chemin local, une ACL ou une permission.
8. Chaque décision sensible déclenche une tentative de journalisation minimale.
   Pour toute opération qui lirait un document ou appellerait Ollama, un journal
   indisponible provoque un refus avant cette lecture ou cet appel.

## Limites connues et évolutions

- L’identité est une simulation libre locale, non un SSO d’entreprise.
- La recherche est lexicale ; elle ne comprend pas encore la similarité sémantique.
- Le chat RAG limite le contexte à trois extraits de 500 caractères.
- La journalisation active couvre toutes les routes qui lisent des documents,
  renvoient une décision ACL ou appellent Ollama. Les routes de session et de
  santé ne sont pas journalisées à ce stade.
- Les documents restent fictifs.
- Un futur MCP devra passer par une passerelle d’actions contrôlées ; Oscar ne
  pourra utiliser que des actions `read` explicitement enregistrées.

## Démarrage et vérification

```text
python3 ui/server.py
curl http://127.0.0.1:3210/healthz
curl http://127.0.0.1:11434/api/version
```

Le premier contrôle vérifie l’API du laboratoire ; le second vérifie Ollama.
