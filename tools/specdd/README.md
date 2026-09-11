# Validation du pilote SpecDD

Cet outillage de développement valide les contrats `.sdd` de ChatPurp.
Il n'est pas une dépendance de l'API ou de l'interface : leur cible reste Rust.

## Versions et périmètre

- CLI officielle `specdd` **1.1.1**, dépendances verrouillées dans
  [package-lock.json](package-lock.json), installation sans scripts de lifecycle
  et avec exclusion des dépendances de développement amont (`--omit=dev`).
- Runtime de référence **Node 22.22.0**, fixé par [.node-version](.node-version).
- Grammaire **1.2** du framework **1.5**, référence Git immuable
  `f5440dc93f4fc75c42f2fbbdfc304bd7ef0e5451` ; voir le
  [langage officiel à cette révision](https://github.com/specdd/specdd/blob/f5440dc93f4fc75c42f2fbbdfc304bd7ef0e5451/LANGUAGE.md).

La référence de langage est consignée dans [package.json](package.json).
Le bootstrap global du framework et ses plugins ne sont pas installés.
Ce pilote adopte la syntaxe et les outils de validation, sans imposer à tous
les fichiers Python une nouvelle hiérarchie de propriété.

Le contrat de répertoire
[SPEC-001](../../specs/001-demo-session-document-read/001-demo-session-document-read.sdd)
est découvert par son nom identique à celui de son dossier. Aucun fichier
racine dépendant du nom du clone n'est nécessaire pour ce pilote. Un futur
déploiement complet de SpecDD devra définir cette racine et ses règles.

## Installation reproductible

Avec le Node indiqué par `.node-version` dans le PATH, depuis la racine :

```sh
npm --prefix tools/specdd ci --omit=dev --ignore-scripts --no-fund --no-audit
npm --prefix tools/specdd run audit
npm --prefix tools/specdd run lint
npm --prefix tools/specdd test
```

`npm ci` utilise exclusivement les versions du lockfile. `node_modules/` est
ignoré par Git ; aucune installation globale de la CLI n'est nécessaire.
`--no-audit` évite un appel implicite au registre pendant l'installation ;
ce n'est pas une déclaration d'absence de vulnérabilités.

La CLI est une dépendance d'exécution de ce petit paquet d'outillage, placé
hors de l'application. Ce classement permet à `--omit=dev` d'exclure les
outils de développement présents dans le shrinkwrap publié par SpecDD.
Une installation incluant ceux-ci a signalé cinq vulnérabilités (dont trois
élevées) le 2026-09-11 ; ils ne sont pas nécessaires au lint. L'installation
retenue les exclut, sans modifier le code officiel de la CLI. Le lockfile
référence encore ces paquets amont non installés et perd leurs indicateurs
`dev` lors de l'intégration du shrinkwrap imbriqué. Un `npm audit --omit=dev`
brut les signale donc encore, même après une installation propre.

`npm run audit` appelle [audit-runtime.mjs](audit-runtime.mjs), qui consulte le
rapport JSON npm complet. Une alerte n'est exclue que si tous ses chemins sont
absents sur disque ET déclarés `dev: true` dans le shrinkwrap officiel installé
de SpecDD. Les exclusions restent visibles dans le résultat. Aucun nom de CVE
n'est ignoré en bloc. Une dépendance installée n'est jamais exclue et une
réponse d'audit invalide/indisponible fait échouer le contrôle. Le 2026-09-11,
le résultat était : zéro alerte active, cinq alertes sur des dépendances de
développement amont absentes. Ce résultat n'est pas un audit vierge de tout
le lockfile.

Sur le Mac du lab, un runtime officiel ARM64 a été téléchargé dans
`.local/specdd-runtime/node-v22.22.0-darwin-arm64/`, hors Git, sans changer le
Node par défaut. Pour cette installation locale :

```sh
PATH="$PWD/.local/specdd-runtime/node-v22.22.0-darwin-arm64/bin:$PATH" npm --prefix tools/specdd test
```

L'archive provient de
[nodejs.org](https://nodejs.org/dist/v22.22.0/node-v22.22.0-darwin-arm64.tar.gz).
Son SHA-256 a été comparé au
[manifeste officiel](https://nodejs.org/dist/v22.22.0/SHASUMS256.txt) :
`5ed4db0fcf1eaf84d91ad12462631d73bf4576c1377e192d222e48026a902640`.
Après clonage sur une autre machine, ce dossier local n'existe pas : fournir
le runtime indiqué avant les commandes npm.

## Contrôles et limites

Le job `SpecDD contracts` du [workflow](../../.github/workflows/tests.yml)
installe Node et les dépendances d'exécution verrouillées, audite cet arbre,
puis lance le lint et les huit
tests de [validation.test.mjs](validation.test.mjs) :

1. La CLI installée correspond à la version déclarée et accepte les specs ;
   les outils de développement amont signalés par l'audit sont absents.
2. La résolution découvre effectivement SPEC-001 et ses 13 paires
   exigence/preuve attendue, chacune présente une fois.
3. Les références explicites du pilote existent et restent dans le dépôt.
4. Une indentation volontairement incorrecte fait échouer la CLI.
5. Une preuve manquante ou une exigence dupliquée fait échouer notre contrôle
   de traçabilité.
6. Une alerte n'est exclue que pour un paquet absent et déclaré dev en amont.
7. Un paquet vulnérable installé bloque même s'il est déclaré dev en amont.
8. Un rapport d'audit indisponible ou des chemins invalides font échouer le contrôle.

La CLI réalise l'analyse syntaxique ; notre test consomme son JSON et ajoute
le contrôle des identifiants du pilote. Ce n'est pas un second parseur `.sdd`.
Une nouvelle exigence discutée implique de mettre à jour le contrat et la
liste attendue dans le test. Cette vérification n'évalue pas le sens d'une
phrase et ne remplace pas les futurs tests de comportement Rust.

L'audit npm est un contrôle de sécurité évolutif qui consulte le registre ;
il est distinct des tests déterministes et fait échouer son étape CI pour une
vulnérabilité haute ou critique. Zéro alerte à une date donnée ne garantit pas
l'absence de toute vulnérabilité.

Les tests invalides utilisent des répertoires temporaires supprimés en fin de
test. Aucun modèle, journal de session réel ou Qdrant local n'est sollicité.
Le résultat du job se vérifie dans GitHub Actions pour le commit concerné ;
une validation locale ne constitue pas à elle seule une réussite distante.

## Conversion et maintenance

Les 13 textes d'exigence et les 13 textes de vérification de l'ancien tableau
Markdown ont été transférés à l'identique. Les scénarios, erreurs, limites et
critères de migration ont été répartis dans les sections du contrat.
L'ancien `spec.md` a été remplacé pour éviter deux sources contradictoires.

Pour mettre à jour l'outillage : choisir une version précise, lire ses changements,
régénérer le lockfile, puis exécuter les contrôles positifs et négatifs. Ne pas
utiliser `latest` dans la CI. La [CLI officielle](https://github.com/specdd/cli)
documente les commandes `lint`, `inspect` et `resolve`.
