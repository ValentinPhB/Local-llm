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
- CI GitHub Actions validée sur `main` : les 35 tests, les contrôles JSON et
  liens Markdown, ainsi que Gitleaks ont réussi sur le commit `38ee851`.
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
- Tests validés : `python3 -m unittest discover -s tests -v` — 50 tests,
  incluant jeton falsifié/expiré, chat sans session et ACL Alice/RH/IT.
- La lecture directe, la vérification ACL, la recherche, le chat simple et le
  RAG écrivent désormais dans `.local/audit/access-decisions.jsonl` une
  décision minimale autorisée ou refusée. La recherche ne conserve ni requête
  ni extrait ; le chat et le RAG ne conservent ni message, ni source, ni réponse.
  Le fichier est hors Git, privé, plafonné à 1 Mo avec une sauvegarde. Si le
  journal est indisponible, l'opération est refusée avant toute lecture ou appel
  Ollama.

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
pas Ollama. La route RAG construit ensuite côté serveur le contexte Ollama avec
ces seuls extraits et renvoie leurs identifiants de source.

Les références à maintenir à chaque évolution sont
`docs/current-request-flow.md` (séquence d’exécution) et
`docs/local-api-detailed-architecture.md` (composants et frontières API).

## Prochaine étape approuvée

Vérifier le résultat de la CI GitHub Actions déclenchée par le commit
`38b88c9`. Le commit RAG `f19a77b` a révélé un test de falsification de jeton
instable : son dernier caractère Base64 pouvait ne modifier que des bits de
remplissage. Le correctif inverse désormais un octet réel de signature ; il a
passé les 50 tests localement et attend sa validation CI. Le socle automatisé
comprend aussi les contrôles JSON, liens Markdown et Gitleaks à chaque push ou
pull request.

La prochaine brique approuvée est le RAG sémantique local : `embeddinggemma`
via Ollama et Qdrant local ARM64, décrits dans `docs/semantic-rag-design.md`.
Le pré-vol a confirmé 56 Go libres et Docker `aarch64`. `embeddinggemma` est
installé (621 Mo) et a produit un vecteur local pour une phrase fictive ; aucun
document n'est indexé. Qdrant ARM64 est démarré, vide, sous `1000:1000`, avec
un volume Docker dédié, 1 Go de RAM, aucune capacité additionnelle et seulement
`127.0.0.1:6333` publié. La prochaine micro-étape est d'ajouter le client
sémantique et ses faux fournisseurs déterministes, avant toute indexation,
puis une CI Qdrant sans recette manuelle. Ne pas ajouter de donnée réelle, MCP,
volume hôte large ou exposition réseau sans décision explicite.

## Reprise recommandée

Dans une nouvelle conversation, ouvrir ce dépôt puis écrire :

```text
Lis STATUS.md et AGENTS.md. Vérifie Git et Ollama, puis reprends la prochaine étape approuvée du LLM Security Lab.
```
