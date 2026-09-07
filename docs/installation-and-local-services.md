# Installation et services locaux

Ce document enregistrera les installations réalisées dans ce laboratoire. Il ne
doit contenir ni secret, ni token, ni mot de passe réel.

## Méthode d'enregistrement

Pour chaque étape, documenter :

1. le composant et son rôle dans l'architecture ;
2. le choix effectué et ses implications de sécurité ;
3. les commandes effectivement exécutées ;
4. les fichiers, données persistantes, processus ou ports créés ;
5. la commande de vérification et son résultat ;
6. la version installée et la date, si utile.

## État actuel

| Élément | État |
| --- | --- |
| Git | Dépôt local initialisé et relié à GitHub ; l'état de synchronisation se vérifie avec `git status --branch` |
| Ollama | Installé, serveur local démarré (version 0.33.3) |
| Modèles Ollama | `qwen3:4b` (2,5 GB) et `embeddinggemma` (621 MB) téléchargés localement |
| Docker / Docker Desktop | Disponible ; Qdrant local ARM64 lancé pour préparer le RAG sémantique |
| Interface locale minimale | Créée et testée ; démarrage manuel nécessaire |
| Annuaire / SSO de démonstration | Fictif, local, jetons signés éphémères ; aucun annuaire d'entreprise |
| Documents de démonstration | 15 fichiers Markdown fictifs, versionnés, classifiés ; RAG lexical actif après ACL |
| Ports réseau du laboratoire | Ollama : `127.0.0.1:11434` ; interface : `127.0.0.1:3210` ; Qdrant : `127.0.0.1:6333` |

## Entrées d'installation

### Ollama — 2026-09-04

- **Rôle :** serveur d'inférence local qui exécutera les modèles et exposera une API locale.
- **Installation :** application installée dans `/Applications/Ollama.app` depuis l'image disque officielle, après vérification de la signature Developer ID et de la notarisation Gatekeeper.
- **Command-line interface :** le raccourci `/usr/local/bin/ollama` a été autorisé lors du premier lancement ; il pointe vers l'exécutable de l'application.
- **Données locales :** Ollama utilise `~/.ollama` pour ses modèles, réglages et journaux. Lors de l'installation initiale, aucun modèle n'était présent ; les modèles ci-dessous sont désormais installés.
- **Réseau :** le processus écoute sur `127.0.0.1:11434`, donc uniquement depuis ce Mac.
- **Mode local uniquement :** `~/.ollama/server.json` contient `"disable_ollama_cloud": true`. Après redémarrage, les journaux confirment `Ollama cloud disabled: true`.
- **Premier modèle :** `qwen3:4b` (ID `359d7dd4bcda`) a été téléchargé localement ; taille indiquée par Ollama : 2,5 GB.
- **Modèle d'embeddings — 2026-09-07 :** `embeddinggemma` (ID `85462619ee72`) est téléchargé localement ; taille indiquée par Ollama : 621 MB. Un appel local `/api/embed` avec une phrase fictive a retourné un vecteur de 768 dimensions. Il n'est pas encore relié à l'API, à Qdrant ou à des documents.
- **Mémoire observée :** lors de la première inférence, le modèle a occupé 3,2 GB via Metal avec un contexte de 4096 tokens. `ollama stop qwen3:4b` l'a déchargé sans supprimer les fichiers sur disque.
- **Thinking :** lors d'un test API, `think: false` n'a pas empêché une trace `<think>` d'apparaître dans `message.content`. Ne pas utiliser ce réglage comme une garantie de non-divulgation.
- **Vérifications :**

  ```bash
  ollama --version
  curl http://127.0.0.1:11434/api/version
  lsof -nP -iTCP:11434 -sTCP:LISTEN
  ```

  Résultat observé : version `0.33.3` ; API opérationnelle ; écoute limitée à `127.0.0.1:11434`.

### Interface locale minimale — 2026-09-04

- **Rôle :** page de conversation servie par `ui/server.py`, sans dépendance tierce ni conteneur.
- **Démarrage :** `python3 ui/server.py`, puis ouvrir `http://127.0.0.1:3210`.
- **Réseau :** le serveur est lié à `127.0.0.1:3210`. Le navigateur lui parle directement ; le serveur appelle uniquement l'endpoint local `http://127.0.0.1:11434/api/chat` pour le chat et le RAG lexical.
- **Réduction de surface :** pas de compte, clé API, import de document, conversation persistante, agent, outil ou MCP.
- **Protection des traces :** le serveur demande `think: false` à Ollama et retire tout contenu précédant `</think>` lorsqu'un modèle renvoie malgré tout une trace balisée.
- **Vérification :** `/healthz` retourne le modèle fixé `qwen3:4b` ; une entrée JSON invalide renvoie `400` ; le test `Réponds exactement : LOCAL-OK` a renvoyé seulement `LOCAL-OK`.

### Simulation SSO locale — 2026-09-07

- **Rôle :** simuler le contrat entre un annuaire d'entreprise et l'API sans
  connecter d'annuaire ni créer de compte réel.
- **Installation :** aucune dépendance, compte, secret persistant ou service
  supplémentaire. Le code Python standard charge
  `config/demo-idp/directory.json` au démarrage de `ui/server.py`.
- **Identités :** Alice, Bob, Charlie et Oscar sont fictifs. Le navigateur les choisit
  explicitement dans le seul but de démonstration ; ce choix n'est pas une
  authentification.
- **Données persistantes :** aucune. Une clé HMAC aléatoire est générée en
  mémoire à chaque démarrage ; tous les cookies de session deviennent invalides
  après redémarrage.
- **Réseau :** aucune nouvelle écoute. Les routes SSO et RBAC font partie du
  processus déjà limité à `127.0.0.1:3210`.
- **Vérification :** `python3 -m unittest discover -s tests -v` teste la
  signature, l'expiration et la falsification de jeton, ainsi que le refus du
  chat sans session et les décisions ACL Alice/RH, Alice/IT et Oscar ciblé.

### Jeu documentaire fictif — 2026-09-07

- **Rôle :** fournir des ressources réelles mais entièrement fictives pour
  tester le lien entre fichier, métadonnées, ACL, lecteur contrôlé et RAG.
- **Installation :** aucun téléchargement ni service. Les fichiers sont livrés
  par Git sous `demo-documents/` : 9 PUBLIC, 3 RH et 3 IT.
- **Configuration :** chaque fichier contient `id`, `classification` et
  `owner`; `config/access-control/demo-policy.json` référence le même
  identifiant, la même classification et le chemin relatif exact.
- **Données persistantes et réseau :** uniquement des fichiers Git ; aucun port
  ajouté ni index vectoriel créé. Leur lecture est uniquement possible après ACL
  et via un chemin déclaré dans la politique ; le chat RAG peut ensuite envoyer
  des extraits autorisés et bornés à Ollama.
- **Vérification :** `tests/test_demo_documents.py` contrôle la répartition
  9/3/3, l'existence de chaque chemin et la cohérence des métadonnées.
- **Lecture contrôlée :** `GET /api/documents/<resource_id>` ne lit le fichier
  qu'après session et ACL ; `tests/test_document_reader.py` et
  `tests/test_local_api.py` vérifient la lecture autorisée, le refus avant
  lecture et le blocage d'un identifiant assimilable à un chemin.

### Qdrant local — 2026-09-07

- **Rôle :** future base vectorielle locale du RAG sémantique. Elle n'est pas
  encore utilisée par l'API et sa collection est vide : aucun document ni
  vecteur n'est indexé à cette étape.
- **Image :** `qdrant/qdrant:v1.19.1-unprivileged`, verrouillée par le digest
  `sha256:801777072776dc81b2ed487571f21ecd30efffd15ddb1671f2193d` dans
  [`compose.qdrant.yml`](../compose.qdrant.yml). Docker Desktop a sélectionné
  l'image ARM64.
- **Persistance :** volume Docker nommé `llm-lab-qdrant-data`, monté seulement
  dans `/qdrant/storage`. Aucun montage du dépôt, du répertoire personnel, de
  la racine macOS ou du socket Docker.
- **Confinement :** utilisateur `1000:1000`, image non privilégiée,
  `cap_drop: ALL`, `no-new-privileges`, limite de 256 processus et 1 Go de RAM.
  Le conteneur ne redémarre pas automatiquement.
- **Réseau :** seul `127.0.0.1:6333` est publié ; aucune exposition LAN ou
  Internet. La santé locale `/healthz` a répondu `healthz check passed`.
- **État applicatif :** les clients sémantiques, l'indexeur contrôlé et le
  writer Qdrant Python sont testés avec des services simulés. Aucune route API
  ne les appelle et le writer n'est pas instancié contre Qdrant réel ; la
  collection reste vide.
