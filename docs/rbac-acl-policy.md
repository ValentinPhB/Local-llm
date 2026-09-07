# Politique RBAC/ACL de démonstration

Le fichier `config/access-control/demo-policy.json` est un jeu de données
fictif utilisé pour concevoir et tester les contrôles d'accès. Il ne crée pas
de comptes et ne fournit pas d'authentification réelle. Il est désormais
utilisé par l'API locale après la vérification d'un jeton de démonstration.

## Décision attendue

La décision est prise côté serveur selon la règle suivante :

```text
si un groupe vérifié est traduit en rôle présent dans allowed_roles : autoriser
sinon : refuser
```

L'absence d'identité vérifiée, de rôle, de ressource ou une politique invalide
entraîne un refus. Aucune phrase du prompt, du document ou du LLM ne participe
à cette décision.

Les groupes viennent de `config/demo-idp/directory.json`, intégré au jeton
signé, puis `group_role_mappings` les traduit : `LAB_READERS -> lab_reader`,
`HR -> rh_reader`, `IT -> it_reader`. Cette séparation reproduit le fait que
l'annuaire possède des groupes tandis que l'application possède ses rôles.

## Matrice attendue

| Identité fictive | PUBLIC | RH | IT |
| --- | --- | --- | --- |
| Alice | Autorisé | Autorisé | Refusé |
| Bob | Autorisé | Refusé | Autorisé |
| Charlie | Autorisé | Refusé | Refusé |
| Oscar | `public-welcome` seulement | Refusé | Refusé |

Cette matrice correspond aux identités et documents fictifs des scénarios
`SEC-01` à `SEC-05`. La récupération lexicale active applique déjà ce filtre
avant toute lecture et recherche ; `POST /api/rag-chat` n'envoie ensuite à
Ollama que les passages autorisés. La future recherche sémantique devra garder
le même ordre de contrôles.

## Ressources physiques de démonstration

La politique référence désormais quinze vrais fichiers sous `demo-documents/` :
neuf PUBLIC, trois RH et trois IT. Chaque ressource contient son `path`, son
`classification` et ses `allowed_roles`; le fichier lui-même porte les mêmes
métadonnées dans son front matter. Le détail est dans
[`demo-document-catalog.md`](demo-document-catalog.md).

Oscar n'a pas le rôle `lab_reader` : le rôle `public_welcome_reader` apparaît
seulement dans l'ACL de `public-welcome`. Son rôle séparé `mcp_read_only` ne
donne aucun droit documentaire. Son contrat pour les MCP futurs est décrit dans
[`mcp-authorization-contract.md`](mcp-authorization-contract.md).

## Limites actuelles

- La lecture directe peut retourner un fichier après autorisation. La
  récupération lexicale et le chat RAG actifs utilisent également ce lecteur,
  mais seuls leurs extraits autorisés et bornés peuvent atteindre Ollama.
- La session est une simulation libre locale, non une identité réelle.
- `GET /api/access-check` retourne seulement autorisé ou refusé. La lecture,
  la récupération lexicale et le RAG sont des routes distinctes déjà actives.

## Moteur déterministe

`decide_access(policy, identity_id, resource_id)` retourne une décision avec :

- `allowed` : booléen d'autorisation ;
- `reason` : `role_match`, `insufficient_role`, `unknown_identity`,
  `unknown_resource` ou `invalid_policy` ;
- `matched_roles` : rôle utilisé lorsqu'une autorisation est accordée.

Le moteur refuse systématiquement une politique invalide, une identité inconnue
ou une ressource inconnue. Les tests standards Python sont dans
`tests/test_access_control.py`. L'API utilise plutôt
`roles_from_groups(...)`, puis `decide_access_for_roles(...)`, car elle reçoit
des groupes après vérification du jeton.

Le lecteur contrôlé qui appelle le moteur est détaillé dans
[`controlled-document-access.md`](controlled-document-access.md).
