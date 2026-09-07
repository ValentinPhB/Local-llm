# Journal d'installation

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
| Git | Dépôt local initialisé et synchronisé avec GitHub |
| Ollama | Installé, serveur local démarré (version 0.33.3) |
| Modèle LLM | `qwen3:4b` téléchargé localement (2,5 GB selon Ollama) |
| Docker / Docker Desktop | Disponible, aucun conteneur du laboratoire lancé |
| Interface locale minimale | Créée et testée ; démarrage manuel nécessaire |
| Annuaire / SSO de démonstration | Fictif, local, jetons signés éphémères ; aucun annuaire d'entreprise |
| Documents de démonstration | 15 fichiers Markdown fictifs, versionnés, classifiés et lisibles après ACL ; aucun RAG |
| Ports réseau du laboratoire | Ollama : `127.0.0.1:11434` ; interface : `127.0.0.1:3210` |

## Entrées d'installation

### Ollama — 2026-09-04

- **Rôle :** serveur d'inférence local qui exécutera les modèles et exposera une API locale.
- **Installation :** application installée dans `/Applications/Ollama.app` depuis l'image disque officielle, après vérification de la signature Developer ID et de la notarisation Gatekeeper.
- **Command-line interface :** le raccourci `/usr/local/bin/ollama` a été autorisé lors du premier lancement ; il pointe vers l'exécutable de l'application.
- **Données locales :** Ollama utilise `~/.ollama` pour ses modèles, réglages et journaux. Aucun modèle n'a encore été téléchargé.
- **Réseau :** le processus écoute sur `127.0.0.1:11434`, donc uniquement depuis ce Mac.
- **Mode local uniquement :** `~/.ollama/server.json` contient `"disable_ollama_cloud": true`. Après redémarrage, les journaux confirment `Ollama cloud disabled: true`.
- **Premier modèle :** `qwen3:4b` (ID `359d7dd4bcda`) a été téléchargé localement ; taille indiquée par Ollama : 2,5 GB. Aucun autre modèle n'est installé.
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
- **Réseau :** le serveur est lié à `127.0.0.1:3210` et transmet uniquement à `http://127.0.0.1:11434/api/chat`.
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
  chat sans session et les décisions ACL Alice/RH et Alice/IT.

### Jeu documentaire fictif — 2026-09-07

- **Rôle :** fournir des ressources réelles mais entièrement fictives pour
  tester le lien entre fichier, métadonnées et ACL avant toute fonctionnalité
  de lecture ou de RAG.
- **Installation :** aucun téléchargement ni service. Les fichiers sont livrés
  par Git sous `demo-documents/` : 9 PUBLIC, 3 RH et 3 IT.
- **Configuration :** chaque fichier contient `id`, `classification` et
  `owner`; `config/access-control/demo-policy.json` référence le même
  identifiant, la même classification et le chemin relatif exact.
- **Données persistantes et réseau :** uniquement des fichiers Git ; aucun port
  ajouté, index créé ou contenu envoyé à Ollama. Leur lecture est uniquement
  possible après ACL et via un chemin déclaré dans la politique.
- **Vérification :** `tests/test_demo_documents.py` contrôle la répartition
  9/3/3, l'existence de chaque chemin et la cohérence des métadonnées.
