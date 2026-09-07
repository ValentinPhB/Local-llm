# Modèle RBAC/ACL de démonstration

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

Cette matrice correspond aux identités et marqueurs fictifs des scénarios
`SEC-01` à `SEC-05`. Lors de la future récupération documentaire, le filtre
sera appliqué avant toute recherche et avant l'envoi de passages au LLM.

## Ressources physiques de démonstration

La politique référence désormais quinze vrais fichiers sous `demo-documents/` :
neuf PUBLIC, trois RH et trois IT. Chaque ressource contient son `path`, son
`classification` et ses `allowed_roles`; le fichier lui-même porte les mêmes
métadonnées dans son front matter. Le détail est dans
[`demo-documents.md`](demo-documents.md).

## Limites actuelles

- Les fichiers existent, mais l'API ne les lit, ne les retourne et ne les
  transmet pas encore à Ollama.
- La session est une simulation libre locale, non une identité réelle.
- La route de démonstration ne retourne aucune donnée de ressource : elle
  montre seulement autorisé ou refusé. Le filtrage de documents avant RAG reste
  la prochaine étape.

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
