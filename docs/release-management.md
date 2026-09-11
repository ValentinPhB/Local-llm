# Release management — CI/CD et qualité des briques

## Objet

Ce document définit le cycle DevOps actuel et sa trajectoire : comment une
modification de code, de politique RBAC, de modèle ou de composant est testée,
analysée, approuvée puis déployée localement.

Il ne décrit pas seulement les versions : une release est un artefact dont la
qualité, la sécurité, la compatibilité et le retour arrière ont été vérifiés.

## État actuel

Une CI GitHub Actions est configurée dans `.github/workflows/tests.yml`. Elle
exécute la suite Python déterministe et Gitleaks à chaque push sur `main` et
pull request. Elle ne déploie rien, ne télécharge aucun modèle et n'appelle pas
Ollama.

Il n'autorise aucun déploiement réseau ou cloud. Les scans de dépendances,
d'images et le SBOM sont des extensions prévues, pas des contrôles actifs.

## Briques et responsabilités de test

| Brique | Code ou artefact | Tests unitaires | Tests d'intégration | Contrôles sécurité |
| --- | --- | --- | --- | --- |
| Interface/API locale | `ui/server.py`, `ui/index.html` | Validation de message, filtre `</think>`, erreurs HTTP | Serveur démarré, session refusée/acceptée, document autorisé/refusé, `/api/chat` avec Ollama simulé | écoute loopback, pas de logs de prompts, analyse Python |
| Simulation SSO | `identity/demo_sso.py`, `directory.json` | signature, expiration, issuer, audience et groupes | cookie falsifié refusé par l'API | aucune clé persistante, cookie HttpOnly, identité libre interdite sur chat |
| Politique d'accès | `demo-policy.json` | groupes -> rôles et décision RBAC/ACL | API + moteur de politique, avant tout contexte LLM | refus par défaut, absence de contournement par prompt |
| Documents fictifs | `demo-documents/` et chemins de politique | présence, nombre, classification et métadonnées | lecteur contrôlé après ACL, avant RAG | aucun document réel, chemin déclaré obligatoire, refus avant lecture |
| Récupération lexicale et RAG | `document_store/retriever.py`, `ui/server.py` | classement lexical, bornage, contexte serveur | ACL avant lecture, extraits autorisés seulement, faux Ollama | pas d'index vectoriel persistant, pas de contenu dans les journaux |
| Ollama | application macOS | hors CI : logiciel tiers | API locale, version, écoute `127.0.0.1` | signature/notarisation, veille CVE avant mise à jour |
| Modèle | manifeste et blobs Ollama | hors CI : artefact tiers | requête non sensible, mémoire et temps de réponse | licence, origine, identifiant de contenu, comportement `think` |
| RAG sémantique en préparation | `semantic_retrieval/` | clients loopback, filtre Qdrant, chunking, indexeur et writer fixe | faux embeddings et faux Qdrant ; Qdrant réel vide | aucune écriture réelle ni route API avant tests d'intégration |
| MCP futur | passerelle et connecteurs | validation des permissions et paramètres | action autorisée/refusée avec faux service | secrets dédiés, moindre privilège, audit, scan dépendances ; Oscar limité à `read` déclaré |

## CI actuelle et extensions prévues

Aujourd'hui, chaque push sur `main` et chaque pull request vers `main` déclenche
deux jobs : la suite Python déterministe et Gitleaks sur l'historique Git. Les
tests Python couvrent aussi la validité JSON et les liens Markdown via les tests
de dépôt. Ils démarrent temporairement l'API sur loopback et simulent Ollama.

Les extensions suivantes ne sont pas encore configurées :

```text
1. Sécurité et supply chain additionnelles
   -> scan des dépendances Python et Node si elles apparaissent
   -> scan d'image si un conteneur est ajouté
   -> génération d'un SBOM pour les artefacts empaquetés

2. RAG sémantique
   -> Qdrant comme service CI éphémère et tests d'intégration automatisés
   -> deux passages fictifs seulement ; aucune recette manuelle comme condition
      de validation
```

Une protection de branche peut ultérieurement imposer ces statuts avant fusion ;
elle n'est pas décrite par ce dépôt.

## Installation des briques dans la CI

La CI installe seulement ce qui est nécessaire pour **tester notre code** sur
un runner isolé. Elle ne doit pas installer l'application macOS Ollama ni
télécharger le modèle réel.

| Brique | Installation dans le runner CI | Raison |
| --- | --- | --- |
| Dépôt | récupération exacte du commit à tester | le commit est l'artefact source |
| Python | version supportée explicitement sélectionnée par le workflow | exécuter analyse et tests de `ui/server.py` |
| Dépendances Python | aucune aujourd'hui : bibliothèque standard uniquement | éviter une dépendance implicite ; plus tard, installer depuis un fichier verrouillé et contrôlé |
| Interface HTML | aucun build ni package aujourd'hui | `index.html` est livré tel quel avec le code source |
| Politique RBAC/ACL | incluse dans le commit ; validée comme JSON | les règles font partie de l'artefact à tester |
| Ollama de test | faux serveur défini par les tests, pas Ollama réel | réponses déterministes, aucun téléchargement de modèle |
| Outils de scan | Gitleaks est exécuté par le workflow ; les scans de dépendances, d'images et le SBOM restent à ajouter | secrets aujourd'hui, supply chain ensuite |

Le workflow actuel récupère donc le commit, installe Python 3.11, lance les
tests puis Gitleaks. Il n'appelle ni le registre d'Ollama ni l'API réelle du
Mac.

## Installation et déploiement local de chaque brique

Le déploiement local applique un commit ou un tag déjà validé par la CI. Toutes
les commandes suivantes sont exécutées manuellement et seulement après accord
explicite : aucun déploiement automatique n'est actif aujourd'hui.

| Brique | Comment elle est installée ou mise à jour sur le Mac | Vérification après installation |
| --- | --- | --- |
| Code du laboratoire | `git fetch origin`, puis sélection du commit/tag validé ; les fichiers Python, HTML et JSON arrivent ensemble | `git status --branch` et commit attendu |
| Python | fourni actuellement par les Command Line Tools de macOS ; aucune librairie tierce n'est installée pour l'interface | `python3 --version`, puis compilation et tests |
| API locale Python | pas d'installation système : démarrage explicite avec `python3 ui/server.py` | `curl http://127.0.0.1:3210/healthz` et écoute `127.0.0.1:3210` |
| Politique RBAC/ACL | fichier JSON livré avec le même commit que le code ; pas de base de données ni migration à ce stade | validation JSON, matrice autorisation/refus et chemins de documents |
| Documents fictifs | fichiers Markdown versionnés avec le code ; aucun téléchargement ou index local | test 9/3/3, métadonnées et chemins référencés par la politique |
| Ollama | application macOS téléchargée depuis la release officielle, montée en lecture seule, signature et notarisation vérifiées, puis copiée dans `/Applications` | `ollama --version`, `/api/version` et écoute `127.0.0.1:11434` |
| Modèle | téléchargement explicite par `ollama pull <nom:tag>` ; les blobs restent dans le stockage Ollama local | `ollama list`, `ollama show`, identifiant de contenu, espace disque et test non sensible |
| RAG sémantique | Qdrant local au digest verrouillé ; clients, indexeur et writer Python versionnés, mais non appelés | tests ACL avant indexation et avant récupération ; aucune donnée réelle |
| MCP futur | connecteur approuvé, version/digest verrouillé, credential dédié injecté hors de Git | tests autorisation/refus et journal d'audit |

### Séquence de déploiement local actuelle

```text
1. Vérifier que le commit/tag a réussi la CI.
2. Mettre à jour le dépôt sur le commit/tag validé.
3. Détecter les briques modifiées dans les notes de release.
4. Mettre à jour seulement ces briques :
   - code/politique : Git ;
   - Ollama : DMG officiel vérifié, si une nouvelle version est prévue ;
   - modèle : ollama pull, si le manifeste cible a changé.
5. Lancer la suite automatisée applicable ; la CI reste la preuve d'acceptation
   du code.
6. Consigner le résultat ou revenir au tag précédent.
```

Une release de code ne réinstalle donc pas automatiquement Ollama ni le modèle.
Chaque artefact externe est mis à jour seulement lorsqu'une version cible est
explicitement décidée et vérifiée.

## Limite volontaire entre CI et CD local

GitHub Actions peut vérifier le code dans le cloud, mais ne peut pas déployer
sur ce Mac sans installer un **runner auto-hébergé** ayant accès à la machine.
Un tel runner pourrait exécuter du code issu des workflows ; il serait donc une
nouvelle surface d'attaque. Nous restons volontairement sur ce modèle :

```text
CI GitHub : teste et publie un résultat
       !=
CD local manuel : l'utilisateur décide d'installer ou mettre à jour le Mac
```

Si nous décidons un jour d'automatiser le CD local, il faudra d'abord concevoir
le runner, son compte système, ses permissions, ses secrets et son isolement.
Ce sera une brique de sécurité à part entière.

## Pourquoi Ollama réel n'est pas exécuté dans la CI

La CI hébergée ne doit ni télécharger un modèle de plusieurs Go ni envoyer de
prompt vers un service extérieur. Les tests d'intégration utiliseront un faux
serveur Ollama qui renvoie des réponses déterministes. Ainsi, la CI teste notre
code et son contrat HTTP, indépendamment de l'inférence.

Une vérification opérationnelle locale du vrai modèle peut confirmer
l'installation, Metal, la mémoire et l'écoute réseau, mais elle ne remplace pas
les tests automatisés et ne constitue pas un critère d'acceptation de code.

## Chaîne CD cible : déploiement local contrôlé

Il n'y a pas de production cloud dans ce laboratoire. Le « déploiement » cible
est donc un déploiement local sur ce Mac, après succès de la CI.

```text
Commit validé sur main
    -> tag de release candidat
    -> notes de release et manifeste des composants
    -> validation manuelle de l'utilisateur
    -> mise à jour locale d'une seule brique
    -> tests automatisés applicables
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
| Preuves locales optionnelles | état des services et vérifications opérationnelles, sans remplacer la CI |

Le modèle Ollama n'est pas intégré au dépôt Git : son nom, son identifiant de
contenu et sa licence sont référencés dans le manifeste de release.

## Promotion et environnements

| Niveau | Objectif | Données autorisées | Déploiement actuel |
| --- | --- | --- | --- |
| Développement | écrire et exécuter les tests | faux uniquement | local, manuel |
| CI | vérifier automatiquement chaque push `main` et pull request | faux uniquement | GitHub Actions actif |
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

1. **Fait :** tests Python déterministes pour RBAC/ACL, SSO, documents,
   lecteur, récupération, RAG, clients sémantiques et indexeur contrôlé.
2. **Fait :** faux Ollama déterministe pour les contrats chat et RAG.
3. **Fait :** workflow GitHub Actions de tests, sans déploiement automatique.
4. **Fait dans le workflow, à confirmer après publication :** Qdrant éphémère
   et test writer + filtre ACL réel, sans document du dépôt.
5. Ajouter scans de dépendances, d'images, SBOM et politique de traitement des CVE dès
   qu'une dépendance, une image ou un MCP est introduit.
6. Créer un manifeste et des notes de release, puis seulement un tag
   `lab-v0.1.0` lorsque le RBAC est réellement appliqué et testé.
