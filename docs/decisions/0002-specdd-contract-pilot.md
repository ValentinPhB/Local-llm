# ADR-0002 — Pilote de contrats au format SpecDD

Date : 2026-09-11. Statut : pilote accepté par l'utilisateur.

## Décision

Convertir le contrat de SPEC-001 de Markdown vers le langage `.sdd` de SpecDD,
avec validation par la CLI officielle et dépendances verrouillées. Préserver
les 13 exigences, leurs identifiants et leurs preuves attendues. Le nom du
contrat reprend celui de son dossier pour être découvert indépendamment du
nom du clone. Les versions et commandes ont une référence unique dans
[l'outillage](../../tools/specdd/README.md).

Les sections `Must` et `Done when` portent exigences et preuves attendues ;
`Tasks` porte les tâches. Le futur plan technique reste en Markdown. README,
guides d'architecture, décisions, AGENTS.md et STATUS.md conservent leur format.

## Limites du pilote

La syntaxe SpecDD et sa découverte sont testées. Le bootstrap global, ses
plugins et une hiérarchie de contrats adjacents à tout le code ne sont pas
déployés. AGENTS.md reste la source des instructions de collaboration.
Les fichiers Rust n'existent pas encore ; leur propriété et leurs dépendances
seront définies dans le plan, sans inférer des permissions système depuis les
sections de la spec.

Le passage à un bootstrap global sera une évolution distincte si le pilote
est utile. Il devra résoudre le nom de la spec racine pour les clones de noms
différents et préserver les règles existantes du lab.

## Raisons et conséquences

La grammaire apporte des sections reconnues par un outil et un contrôle
syntaxique exploitable en CI. Notre test de traçabilité évite la disparition
silencieuse d'un identifiant d'exigence ou de sa preuve attendue.
Ni ce format ni son lint ne prouvent que le futur programme Rust respecte les
exigences ; les tests fonctionnels restent nécessaires.

La CLI requiert Node pour l'outillage uniquement. Le code applicatif conserve
la cible Rust de [ADR-0001](0001-rust-sdd-and-deployment-boundaries.md).
L'ancien contrat Markdown est remplacé, plutôt que maintenu en double.

## Vérification

Syntaxe officielle, découverte du contrat, 13 paires exigence/preuve,
références locales et tests négatifs du validateur/contrôle de traçabilité.
Ces contrôles sont exécutables localement et ajoutés à GitHub Actions ; une
configuration CI locale ne constitue pas une preuve de succès distant.
