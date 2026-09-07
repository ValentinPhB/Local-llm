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
- La politique est un contrat de test ; son moteur d'évaluation sera ajouté à
  l'étape suivante.
