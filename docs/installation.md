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
| Chatbox | Installé, configuration Ollama à réaliser |
| Ports réseau du laboratoire | Ollama : `127.0.0.1:11434` uniquement |

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

### Chatbox Community Edition — 2026-09-04

- **Rôle :** client local natif de conversation. Il remplacera l'interface web conteneurisée initialement envisagée.
- **Choix de sécurité :** cette option évite d'ajouter une interface web conteneurisée avec ses dépendances. Cela ne rend pas le client automatiquement sûr : aucune fonction cloud, recherche web, import documentaire, agent ou MCP ne sera configurée.
- **Installation :** application `Chatbox.app` version `1.22.3` installée dans `/Applications` depuis le DMG Apple Silicon référencé par la release officielle.
- **Intégrité :** SHA-256 du DMG : `2dbee431934730a7d089e32a7b8f4fbfc52b3b3bc19bdcb8ccfbebdc1732060f`.
- **Signature :** `codesign` a validé l'application ; Gatekeeper l'a acceptée comme `Notarized Developer ID`, signée par `benn huang (YJ5GSB3AMW)`.
- **Réseau prévu :** uniquement `http://127.0.0.1:11434` vers Ollama. Aucun compte ni clé API ne sera utilisé.
- **État :** l'application n'a pas encore été lancée ni configurée.
