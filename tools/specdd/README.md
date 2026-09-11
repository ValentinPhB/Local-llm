# Validation des contrats SpecDD

Outillage de développement, pas une dépendance de l’API ou du frontend.
CLI officielle specdd 1.1.1, Node 22.23.2, grammaire 1.2 du framework 1.5.
La référence immuable f5440dc93f4fc75c42f2fbbdfc304bd7ef0e5451 est fixée dans
[package.json](package.json) ; les paquets sont verrouillés dans
[package-lock.json](package-lock.json).

Le bootstrap global et ses plugins ne sont pas installés. Les contrats portent
le nom de leur dossier : SPEC-001 et SPEC-002 sont découverts quel que soit le
nom du clone. Les règles de collaboration restent dans AGENTS.md.
Les sections .sdd ne créent pas des permissions système.

## Installation et contrôles

Après installation du [Node dédié](../browser-tests/runtime-lock.json),
depuis la racine du dépôt :

~~~sh
export PATH="$PWD/.local/browser-runtime/node-v22.23.2-darwin-arm64/bin:$PATH"
npm --prefix tools/specdd ci --omit=dev --ignore-scripts --no-fund --no-audit
npm --prefix tools/specdd run audit
npm --prefix tools/specdd run lint
npm --prefix tools/specdd test
~~~

Aucune CLI globale. --no-audit évite l’audit implicite de l’installation,
pas l’audit explicite suivant. --omit=dev exclut les outils de développement
amont qui figurent dans le shrinkwrap publié par SpecDD mais ne sont pas
nécessaires à la validation.

## Particularité de l’audit

Le lockfile intègre les dépendances dev amont sans toujours conserver leurs
indicateurs. Un npm audit brut peut donc signaler des paquets non installés.
[audit-runtime.mjs](audit-runtime.mjs) consulte le résultat complet : une alerte
n’est exclue que si tous ses chemins sont absents ET déclarés dev dans le
shrinkwrap officiel installé. Les exclusions restent visibles ; aucun numéro
de CVE n’est ignoré globalement. Une dépendance installée n’est jamais exclue.

Cette politique n’équivaut pas à un lockfile intégral sans alerte. Les tests
couvrent les exclusions, dépendances installées, chemins invalides et réponses
d’audit indisponibles. Les alertes hautes/critiques actives bloquent la CI.
Le résultat daté d’un audit n’est pas une garantie de sécurité future.

## Preuves de syntaxe, pas de comportement

Les neuf tests de [validation.test.mjs](validation.test.mjs) vérifient :

- Version installée et lint officiel.
- Découverte et 14 paires exigence/preuve de SPEC-001.
- Découverte et 7 paires de SPEC-002.
- Références explicites locales et confinées au dépôt.
- Rejet d’une indentation mal formée.
- Rejet d’identifiants manquants, dupliqués ou inattendus.
- Trois tests de politique d’audit.

Notre code consomme le JSON de la CLI ; il ne constitue pas un second parseur
.sdd. Ces contrôles ne prouvent pas l’implémentation de l’application :
les tests Rust/HTTP/navigateur sont décrits dans la
[stratégie de preuve](../../docs/security-test-strategy.md).

Une évolution de comportement conserve ou ajuste explicitement les identifiants
Must / Done when, puis ajoute les tests applicatifs correspondants.
Les tâches vivent dans Tasks ; les plans et guides restent en Markdown.
