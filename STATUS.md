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
  `127.0.0.1:3210`, sans compte réel, lecture documentaire, RAG, agent, outil
  ou MCP.
- Le serveur fixe le modèle à `qwen3:4b`, n'enregistre pas les messages et
  retire le préfixe de raisonnement Qwen jusqu'à `</think>` lorsqu'il apparaît.
- Test validé : `Réponds exactement : LOCAL-OK` a affiché seulement
  `LOCAL-OK` dans l'interface.
- Simulation SSO locale : annuaire fictif dans `config/demo-idp/directory.json`;
  Alice, Bob, Charlie et Oscar reçoivent un JWT signé, valable 15 minutes,
  placé dans un cookie `HttpOnly`. La clé est aléatoire, uniquement en mémoire
  et un redémarrage invalide les sessions.
- L'API vérifie le jeton avant le chat. Elle transforme ses groupes en rôles
  puis expose seulement une vérification ACL. Ce n'est pas une authentification
  : l'identité est volontairement choisie librement.
- Jeu documentaire : quinze fichiers Markdown fictifs existent dans
  `demo-documents/` (9 PUBLIC, 3 RH, 3 IT). Leur chemin et leurs métadonnées
  correspondent à la politique ACL. L'API les lit seulement après autorisation,
  via le chemin déclaré et confiné à `demo-documents/`, et ne les envoie jamais
  à Ollama.
- Oscar est autorisé seulement sur `public-welcome`. Son rôle `mcp_read_only`
  est un contrat pour les futurs MCP approuvés : actions `read` explicitement
  enregistrées seulement ; toute autre action est refusée. Aucun MCP n'est
  installé à ce stade.
- Tests validés : `python3 -m unittest discover -s tests -v` — 28 tests,
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

Le contrat identité -> groupes -> rôles -> ACL fonctionne en simulation locale.
Le lecteur contrôlé retourne un document uniquement après autorisation et ne
lit pas de fichier en cas de refus. La récupération lexicale ne classe que les
documents autorisés, retourne trois extraits bornés au maximum et ne contacte
pas Ollama.

## Prochaine étape approuvée

Implémenter la génération augmentée conçue dans
`docs/augmented-generation-design.md` : route distincte du chat simple,
contexte construit exclusivement côté serveur à partir des extraits ACL, sources
utilisées retournées et tests prouvant qu'aucun passage interdit ne peut
atteindre Ollama. Ne pas ajouter de MCP, de donnée réelle, d'embeddings ou de
base vectorielle.

## Reprise recommandée

Dans une nouvelle conversation, ouvrir ce dépôt puis écrire :

```text
Lis STATUS.md et AGENTS.md. Vérifie Git et Ollama, puis reprends la prochaine étape approuvée du LLM Security Lab.
```
