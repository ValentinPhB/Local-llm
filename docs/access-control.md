# Modèle RBAC/ACL de démonstration

Le fichier `config/access-control/demo-policy.json` est un jeu de données
fictif utilisé pour concevoir et tester les contrôles d'accès. Il ne crée pas
de comptes, ne fournit pas d'authentification et n'est pas encore utilisé par
l'interface locale.

## Décision attendue

La décision est prise côté serveur selon la règle suivante :

```text
si l'identité possède au moins un rôle présent dans allowed_roles : autoriser
sinon : refuser
```

L'absence d'identité, de rôle ou de ressource entraîne un refus. Aucune phrase
du prompt, du document ou du LLM ne participe à cette décision.

## Matrice attendue

| Identité fictive | PUBLIC | RH | IT |
| --- | --- | --- | --- |
| Alice | Autorisé | Autorisé | Refusé |
| Bob | Autorisé | Refusé | Autorisé |
| Charlie | Autorisé | Refusé | Refusé |

Cette matrice correspond aux identités et marqueurs fictifs des scénarios
`SEC-01` à `SEC-05`. Lors de la future récupération documentaire, le filtre
sera appliqué avant toute recherche et avant l'envoi de passages au LLM.

## Limites actuelles

- Il n'existe pas encore de connexion utilisateur ni de session.
- Les ressources ne sont pas encore des fichiers ou des documents importés.
- Le moteur `access_control/engine.py` évalue maintenant la politique sous
  forme de fonction interne. Il n'est pas encore relié à une session ou à
  l'interface HTTP.

## Moteur déterministe

`decide_access(policy, identity_id, resource_id)` retourne une décision avec :

- `allowed` : booléen d'autorisation ;
- `reason` : `role_match`, `insufficient_role`, `unknown_identity`,
  `unknown_resource` ou `invalid_policy` ;
- `matched_roles` : rôle utilisé lorsqu'une autorisation est accordée.

Le moteur refuse systématiquement une politique invalide, une identité inconnue
ou une ressource inconnue. Les tests standards Python sont dans
`tests/test_access_control.py`.
