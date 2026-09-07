# Architecture détaillée de l’API locale

## Rôle

L’API du laboratoire est le processus `python3 ui/server.py`. Elle écoute sur
`127.0.0.1:3210`, sert l’interface et applique les contrôles de sécurité avant
tout accès à un document ou appel à Ollama.

Elle ne modifie pas Ollama : elle utilise son API native locale sur
`127.0.0.1:11434`.

```text
Navigateur -- 127.0.0.1:3210 --> API Python -- 127.0.0.1:11434 --> Ollama
```

## Composants

| Fichier | Responsabilité |
| --- | --- |
| `ui/server.py` | Routes HTTP, validation, sessions, orchestration ACL/RAG et relais Ollama. |
| `ui/index.html` | Interface locale ; n’est jamais une source de permission. |
| `identity/demo_sso.py` | Annuaire fictif, émission et vérification du JWT court. |
| `access_control/engine.py` | Traduction groupes -> rôles et décision RBAC/ACL déterministe. |
| `document_store/reader.py` | Lecture confinée au chemin déclaré, après autorisation. |
| `document_store/retriever.py` | Classement lexical des seuls documents déjà autorisés. |
| `config/demo-idp/directory.json` | Quatre identités et leurs groupes fictifs. |
| `config/access-control/demo-policy.json` | Groupes, rôles, ACL et chemins des 15 documents. |

## Routes

| Route | Rôle | Accès documentaire / Ollama |
| --- | --- | --- |
| `GET /healthz` | Vérifie que l’API répond. | Ni document ni Ollama. |
| `POST /api/demo-session` | Crée la session fictive signée. | Ni document ni Ollama. |
| `GET /api/session` | Vérifie et décrit la session. | Ni document ni Ollama. |
| `GET /api/access-check` | Retourne une décision ACL. | Ne lit pas de document. |
| `GET /api/documents/<id>` | Lit un document autorisé. | Document seulement. |
| `POST /api/retrieve` | Retourne des extraits autorisés. | Documents autorisés seulement. |
| `POST /api/chat` | Chat simple. | Ollama, sans document. |
| `POST /api/rag-chat` | Chat avec extraits autorisés. | Documents autorisés puis Ollama. |
| `POST /api/logout` | Supprime le cookie côté navigateur. | Ni document ni Ollama. |

## État et données

- La clé JWT est créée aléatoirement au démarrage et reste seulement en mémoire.
- Les sessions expirent après 15 minutes et un redémarrage les invalide.
- Les prompts et réponses ne sont pas journalisés par le serveur.
- Les documents sont des fichiers Markdown fictifs versionnés dans Git.
- Aucun index persistant, embedding, base vectorielle, conversation ou MCP n’existe.

## Contrôles de frontière

1. Le navigateur ne choisit jamais ses rôles ou les sources RAG.
2. L’API vérifie le jeton avant toute route protégée.
3. L’ACL est évaluée avant la lecture physique d’un fichier.
4. Le lecteur refuse les chemins client et reste dans `demo-documents/`.
5. Le récupérateur reçoit uniquement des identifiants déjà autorisés.
6. Le chat RAG construit son contexte côté serveur, puis retourne les sources.
7. Ollama ne reçoit jamais un chemin local, une ACL ou une permission.

## Limites connues et évolutions

- L’identité est une simulation libre locale, non un SSO d’entreprise.
- La recherche est lexicale ; elle ne comprend pas encore la similarité sémantique.
- Le chat RAG limite le contexte à trois extraits de 500 caractères.
- Les documents restent fictifs.
- Un futur MCP devra passer par une passerelle d’actions contrôlées ; Oscar ne
  pourra utiliser que des actions `read` explicitement enregistrées.
