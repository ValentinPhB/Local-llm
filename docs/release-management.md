# Release management — CI/CD et qualité des briques

## Objet

Ce document définit le futur cycle DevOps du laboratoire : comment une
modification de code, de politique RBAC, de modèle ou de composant est testée,
analysée, approuvée puis déployée localement.

Il ne décrit pas seulement les versions : une release est un artefact dont la
qualité, la sécurité, la compatibilité et le retour arrière ont été vérifiés.

## État actuel

**Aucune CI/CD n'est encore configurée.** Le dépôt ne contient pas de workflow
GitHub Actions et les tests actuels sont exécutés localement et manuellement.

Ce document est la cible à atteindre avant de considérer le laboratoire comme
reproductible. Il n'autorise aucun déploiement réseau ou cloud.

## Briques et responsabilités de test

| Brique | Code ou artefact | Tests unitaires | Tests d'intégration | Contrôles sécurité |
| --- | --- | --- | --- | --- |
| Interface/API locale | `ui/server.py`, `ui/index.html` | Validation de message, filtre `</think>`, erreurs HTTP | Serveur démarré, `/healthz`, `/api/chat` avec Ollama simulé | écoute loopback, pas de logs de prompts, analyse Python |
| Politique d'accès | `demo-policy.json` | Décision RBAC/ACL pour chaque identité/ressource | API + moteur de politique, avant l'appel LLM | refus par défaut, absence de contournement par prompt |
| Ollama | application macOS | hors CI : logiciel tiers | API locale, version, écoute `127.0.0.1` | signature/notarisation, veille CVE avant mise à jour |
| Modèle | manifeste et blobs Ollama | hors CI : artefact tiers | requête non sensible, mémoire et temps de réponse | licence, origine, identifiant de contenu, comportement `think` |
| RAG futur | index, métadonnées et récupérateur | filtre ACL, extraction et chunking | aucun passage interdit envoyé au LLM | tests d'isolation utilisateur et injection documentaire |
| MCP futur | passerelle et connecteurs | validation des permissions et paramètres | action autorisée/refusée avec faux service | secrets dédiés, moindre privilège, audit, scan dépendances |

## Chaîne CI cible

Chaque branche de travail et chaque pull request vers `main` devra déclencher :

```text
1. Validation
   -> syntaxe Python et HTML
   -> JSON valide pour les politiques et configurations
   -> formatage et contrôle statique

2. Tests unitaires
   -> moteur RBAC/ACL : matrice complète autorisation/refus
   -> API : validations, erreurs et filtre de raisonnement

3. Tests d'intégration isolés
   -> serveur Python démarré temporairement sur loopback
   -> faux serveur Ollama : aucune inférence réelle ni appel cloud
   -> contrats HTTP : healthcheck, chat, erreurs et délais

4. Sécurité et supply chain
   -> détection de secrets dans Git
   -> scan des dépendances Python et Node si elles apparaissent
   -> scan d'image si un conteneur est ajouté
   -> génération d'un SBOM pour les artefacts empaquetés

5. Publication du résultat CI
   -> statut succès/échec attaché au commit
   -> rapports de tests et scans conservés comme artefacts CI
```

Un échec à une étape bloque la fusion. Une exception exige une décision de
risque explicite, tracée dans la pull request et la note de release.

## Pourquoi Ollama réel n'est pas exécuté dans la CI

La CI hébergée ne doit ni télécharger un modèle de plusieurs Go ni envoyer de
prompt vers un service extérieur. Les tests d'intégration utiliseront un faux
serveur Ollama qui renvoie des réponses déterministes. Ainsi, la CI teste notre
code et son contrat HTTP, indépendamment de l'inférence.

Les tests avec le vrai modèle restent des **tests de qualification locale** sur
le Mac : ils valident l'installation, Metal, la mémoire, le comportement réel
de Qwen et l'absence d'exposition réseau.

## Chaîne CD cible : déploiement local contrôlé

Il n'y a pas de production cloud dans ce laboratoire. Le « déploiement » cible
est donc un déploiement local sur ce Mac, après succès de la CI.

```text
Commit validé sur main
    -> tag de release candidat
    -> notes de release et manifeste des composants
    -> validation manuelle de l'utilisateur
    -> mise à jour locale d'une seule brique
    -> tests de qualification locale
    -> tag de release final ou rollback
```

La validation manuelle est obligatoire avant toute action qui :

- télécharge ou supprime un modèle ;
- installe ou met à jour une application tierce ;
- ajoute un document, une persistance, un compte ou un MCP ;
- modifie une exposition réseau ;
- introduit un secret ou un identifiant de service.

## Artefacts d'une release

Une release devra produire ou référencer :

| Artefact | Contenu |
| --- | --- |
| Tag Git signé ou identifié | code exact livré |
| Notes de release | changements, risques, incompatibilités et rollback |
| Rapports CI | tests, qualité, scans et versions d'outils |
| Manifeste | versions Ollama/modèle, hash de la politique, dépendances |
| SBOM si dépendances ou image | inventaire exploitable pour les CVE |
| Preuves locales | commandes et résultats de qualification sur le Mac |

Le modèle Ollama n'est pas intégré au dépôt Git : son nom, son identifiant de
contenu et sa licence sont référencés dans le manifeste de release.

## Promotion et environnements

| Niveau | Objectif | Données autorisées | Déploiement actuel |
| --- | --- | --- | --- |
| Développement | écrire et exécuter les tests | faux uniquement | local, manuel |
| CI | vérifier automatiquement chaque changement | faux uniquement | à créer |
| Qualification locale | vérifier le vrai Ollama et le vrai modèle | prompts non sensibles uniquement | local, manuel |
| Release locale | état validé et documenté | faux, puis données explicitement autorisées | à créer |

Un futur environnement de démonstration ou de production ne sera pas déduit de
ce tableau : il nécessitera une architecture, une authentification, une gestion
des secrets et une décision d'exposition propres.

## Retour arrière

Le pipeline doit préparer le retour arrière avant une release :

- code/politique : revenir au tag Git précédent et redémarrer l'API locale ;
- dépendance : restaurer la version verrouillée précédente ;
- image future : redéployer le digest précédemment validé ;
- modèle : conserver l'identifiant de contenu précédent et ne supprimer aucun
  blob sans décision explicite ;
- MCP futur : désactiver le connecteur et révoquer son credential dédié.

Un rollback restaure un état logiciel ; il ne supprime pas silencieusement les
données ou modèles créés pendant la release.

## Plan d'implémentation CI/CD

1. Ajouter des tests Python déterministes pour le moteur RBAC/ACL.
2. Extraire l'appel Ollama derrière une interface testable et créer un faux
   serveur Ollama pour les tests d'intégration.
3. Ajouter un workflow GitHub Actions : validation, tests et détection de
   secrets ; sans déploiement automatique.
4. Ajouter scans de dépendances, SBOM et politique de traitement des CVE dès
   qu'une dépendance, une image ou un MCP est introduit.
5. Créer un manifeste et des notes de release, puis seulement un tag
   `lab-v0.1.0` lorsque le RBAC est réellement appliqué et testé.
