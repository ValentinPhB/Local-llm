# État de reprise — LLM Security Lab

Mettre à jour ce fichier à la fin de chaque séance significative. Il est le
point de reprise du projet, pas un journal exhaustif.

## Dernière mise à jour

2026-09-04

## Objectif

Construire progressivement un laboratoire LLM local et sécurisé sur Mac Apple
Silicon. Les couches futures suivront toujours cet ordre : identité, décision
d'accès RBAC/ACL, récupération documentaire filtrée, puis LLM et MCP.

## État validé

- Dépôt Git : branche `main`, synchronisée avec `origin/main`.
- Ollama 0.33.3 : API liée à `127.0.0.1:11434` ; cloud Ollama désactivé.
- Modèle local : `qwen3:4b`, environ 2,5 GB sur disque ; environ 3,2 GB en
  mémoire pendant une inférence.
- Interface locale minimale : `ui/server.py`, servie uniquement sur
  `127.0.0.1:3210`, sans compte, document, RAG, agent, outil ou MCP.
- Le serveur fixe le modèle à `qwen3:4b`, n'enregistre pas les messages et
  retire le préfixe de raisonnement Qwen jusqu'à `</think>` lorsqu'il apparaît.
- Test validé : `Réponds exactement : LOCAL-OK` a affiché seulement
  `LOCAL-OK` dans l'interface.

## À connaître au redémarrage

- L'interface n'est pas un service permanent. Pour la démarrer :

  ```text
  python3 ui/server.py
  ```

  Puis ouvrir `http://127.0.0.1:3210`.
- Vérifier Ollama avant usage :

  ```text
  curl http://127.0.0.1:11434/api/version
  ollama list
  ```

- Ne pas utiliser Chatbox : les versions testées s'arrêtent immédiatement sur
  ce Mac. Elles ne font pas partie de l'architecture du laboratoire.
- Ne jamais ajouter de données réelles, secrets, documents personnels ou
  credentials dans le dépôt ou les tests.

## Prochaine étape approuvée

Préparer des identités, rôles et ressources entièrement fictifs pour concevoir
la première couche RBAC/ACL. Ne pas encore importer de document réel ni ajouter
de MCP.

## Reprise recommandée

Dans une nouvelle conversation, ouvrir ce dépôt puis écrire :

```text
Lis STATUS.md et AGENTS.md. Vérifie Git et Ollama, puis reprends la prochaine étape approuvée du LLM Security Lab.
```
