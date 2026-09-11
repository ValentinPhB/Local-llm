# Spécifications et méthode de développement

## Direction retenue

Depuis le 11 septembre 2026, le laboratoire évolue par spécifications (SDD),
avec des responsabilités métier explicites (DDD), vers du code applicatif Rust.
La décision et ses limites sont dans
[ADR-0001](../docs/decisions/0001-rust-sdd-and-deployment-boundaries.md).
L'application exécutée reste actuellement en Python.

Le pilote utilise maintenant le format `.sdd` de SpecDD. La syntaxe de
référence et les versions sont documentées dans
[l'outillage de validation](../tools/specdd/README.md) et
[ADR-0002](../docs/decisions/0002-specdd-contract-pilot.md).
La première spec est un contrat de répertoire : son nom reprend celui du
dossier qui la contient. Le nom du clone (`LLM-perso` localement, `ChatPurp`
sur GitHub) ne change donc pas sa découverte.

Le bootstrap global SpecDD n'est pas installé dans ce pilote. `AGENTS.md`
reste le point d'entrée des règles de collaboration ; le `.sdd` décrit le
comportement à réaliser, sans prétendre attribuer les fichiers Rust à venir.

## Faire avec l'utilisateur

Chaque étape porte sur un changement cohérent. Expliquer le besoin, les choix
et les preuves attendues ; laisser à l'utilisateur les arbitrages métier.
L'objectif est sa montée en compétence : présenter les notions et les fichiers
avant d'agir, expliquer le résultat avec un exemple du lab, puis lui laisser
la main avant la prochaine étape pédagogique. Une suite d'exécutions suivie
d'un compte rendu final ne remplace pas cet accompagnement.
Son accord sur la direction ne signifie pas que tous les détails futurs sont
déjà validés. Préparer des propositions concrètes, signaler les ambiguïtés et
tenir compte des décisions déjà prises sans redemander les mêmes accords.

## Cycle d'une fonctionnalité

1. **Spécification** : décrire le besoin, les acteurs, les exigences identifiées,
   les erreurs, les effets interdits et le hors-périmètre dans le contrat `.sdd`.
2. **Clarification** : résoudre les ambiguïtés qui changent les droits ou le
   comportement attendu avec l'utilisateur ; consigner les décisions.
3. **Plan** : décrire dans `plan.md` les composants, contrats HTTP, dépendances,
   stratégie de test et migration. Les bibliothèques et versions sont choisies
   à cette étape, après vérification de leur compatibilité.
4. **Tâches** : utiliser la section `Tasks` du `.sdd` pour des changements courts
   reliés aux exigences ; ne pas maintenir une seconde liste dans `tasks.md`.
5. **Implémentation et preuves** : écrire les tests de comportement puis le
   code, exécuter les contrôles appropriés et relever les limites restantes.
6. **Documentation** : mettre à jour les explications de ce qui fonctionne et
   relier chaque exigence à ses tests et aux résultats réellement obtenus.

Le fichier `plan.md` est créé quand son étape commence ; il explique les choix
techniques sans recopier les exigences. Aucun fichier vide ni case cochée sans
preuve. La CLI SpecDD sert à valider et découvrir les contrats ; Spec Kit
n'est pas utilisé.

## Statut des spécifications

Une spec passe de **proposée** à **validée**, puis **en implémentation** et
**vérifiée**. Une spec vérifiée reste maintenue lors des changements suivants.
Un résultat local et un résultat CI distant sont des preuves distinctes :
ne jamais présenter une CI non exécutée comme réussie.

Les tests actuels servent de référence, pas de définition automatique du bon
comportement. La conformité repose sur des exigences discutées et des tests
indépendants de la structure du code. Les recettes d'acceptation sont
automatisées, y compris les scénarios navigateur quand l'interface est migrée.
La discussion humaine des règles n'est pas une recette manuelle.

## Où écrire quoi ?

| Emplacement | Source de référence pour |
| --- | --- |
| `specs/<fonctionnalité>/<fonctionnalité>.sdd` | Contrat du pilote, exigences, preuves attendues et tâches |
| `specs/<fonctionnalité>/plan.md` | Choix techniques et trajectoire de mise en œuvre |
| `docs/decisions/` | Décisions d'architecture, raisons et conséquences |
| `docs/` | Architecture et exploitation effectives, avec futurs explicitement marqués |
| `AGENTS.md` | Instructions de collaboration et règles de sécurité transverses |
| `STATUS.md` | Reprise locale, ignorée par Git, jamais seule source d'une décision produit |

Faire des liens vers les références plutôt que recopier les mêmes contrats.
Un changement de comportement commence par la mise à jour de sa spec ; un
correctif rétablissant le contrat existant conserve ce contrat et ajoute les
preuves de non-régression utiles.

## Première fonctionnalité

| Spécification | Statut | Implémentation Rust |
| --- | --- | --- |
| [001 — Session de démonstration et lecture autorisée](001-demo-session-document-read/001-demo-session-document-read.sdd) | Contrat proposé ; conversion SpecDD validée localement | Non commencée |

Les 13 identifiants sont conservés dans `Must` (exigences) et `Done when`
(preuves attendues). Les contrôles de l'outillage vérifient la syntaxe,
la découverte, les références et cette correspondance, y compris des cas
invalides. Trois tests supplémentaires vérifient le traitement de l'audit des
dépendances, soit huit tests de l'outillage au total. Ils ne prouvent pas encore
la conformité de l'application Rust.
Les statuts locaux ne valent pas exécution réussie du job GitHub Actions.

## Vocabulaire commun

- **Identité vérifiée** : identité issue d'un jeton validé par le serveur ;
  dans ce lab, le choix initial de l'identité est libre et fictif.
- **Groupe / rôle** : attribut d'identité / permission applicative obtenue par
  traduction côté serveur ; ce ne sont pas des paramètres choisis par le client.
- **Ressource** : document déclaré par son identifiant dans la politique.
- **Classification** : étiquette PUBLIC, RH ou IT ; elle ne suffit pas à
  autoriser l'accès (Oscar n'a pas accès à tous les documents PUBLIC).
- **Décision d'accès** : résultat déterministe de la politique RBAC/ACL.
- **Audit** : trace minimale d'une décision ; une autorisation journalisée ne
  prouve pas que la lecture a ensuite réussi.
- **Passage / index** : extrait d'un document / représentation dérivée pour
  la recherche, jamais source souveraine des droits.
- **Crate** : unité Rust de compilation ; sa séparation n'isole pas les
  permissions du système d'exploitation.
- **Service** : processus joignable et exploité indépendamment ; un dépôt Git
  peut contenir plusieurs services.
