# État de reprise — LLM Security Lab

Mettre à jour ce fichier à la fin de chaque séance significative. Il est le
point de reprise du projet, pas un journal exhaustif.

## Dernière mise à jour

2026-09-07

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
  `127.0.0.1:3210`, sans compte réel, document, RAG, agent, outil ou MCP.
- Le serveur fixe le modèle à `qwen3:4b`, n'enregistre pas les messages et
  retire le préfixe de raisonnement Qwen jusqu'à `</think>` lorsqu'il apparaît.
- Test validé : `Réponds exactement : LOCAL-OK` a affiché seulement
  `LOCAL-OK` dans l'interface.
- Simulation SSO locale : annuaire fictif dans `config/demo-idp/directory.json`;
  Alice, Bob et Charlie reçoivent un JWT signé, valable 15 minutes, placé dans
  un cookie `HttpOnly`. La clé est aléatoire, uniquement en mémoire et un
  redémarrage invalide les sessions.
- L'API vérifie le jeton avant le chat. Elle transforme ses groupes en rôles
  puis expose seulement une vérification ACL de ressources fictives. Ce n'est
  pas une authentification : l'identité est volontairement choisie librement.
- Tests validés : `python3 -m unittest discover -s tests -v` — 15 tests,
  incluant jeton falsifié/expiré, chat sans session et ACL Alice/RH/IT.

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

## Étape en cours

Le contrat identité -> groupes -> rôles -> ACL fonctionne en simulation locale
mais ne protège encore aucun document : les ressources sont seulement des
identifiants fictifs et la route de vérification ne renvoie aucune donnée.

## Prochaine étape à décider avec l'utilisateur

Concevoir un premier jeu de documents strictement fictifs, leurs métadonnées
de classification et leur import contrôlé. Avant tout code, définir le format,
le stockage local, les tests de refus et la garantie qu'aucun passage interdit
n'est récupéré ou envoyé à Ollama. Ne pas ajouter de MCP ni de donnée réelle.

## Reprise recommandée

Dans une nouvelle conversation, ouvrir ce dépôt puis écrire :

```text
Lis STATUS.md et AGENTS.md. Vérifie Git et Ollama, puis reprends la prochaine étape approuvée du LLM Security Lab.
```
