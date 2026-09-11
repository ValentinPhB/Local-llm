# Tests navigateur de l'application Rust

Node 22.23.2, Puppeteer Core 25.10.0 et Chromium Headless Shell 153.0.8010.36
sont dédiés au dépôt. Ils ne sont pas des dépendances de l'API déployée.
Les versions/URLs/empreintes sont dans [runtime-lock.json](runtime-lock.json).

## Installation et exécution

Voir [installation et CI](../../docs/release-management.md). L'installateur
[install-runtime.mjs](install-runtime.mjs) contrôle les archives puis les
exécutables avant installation dans .local. npm ci --ignore-scripts installe
seulement l'outillage verrouillé, sans télécharger un autre navigateur.

~~~sh
sh tools/browser-tests/run.sh policy
sh tools/browser-tests/run.sh audit
sh tools/browser-tests/run.sh browser
sh tools/check.sh
~~~

policy : tests hors ligne des garde-fous/audits. audit : vérifications réseau
npm et OSV. browser : navigateur réel, page synthétique, erreur volontaire
et nettoyage. check.sh construit aussi le vrai frontend puis exécute app :
quatre identités, documents autorisés/refusés, recherche, chat/RAG, HTML inerte,
cookie HttpOnly et déconnexion. Aucun appel à Ollama ni au Qdrant du lab.

## Protections des fixtures

[launch-policy.mjs](launch-policy.mjs) vérifie versions, chemins et empreintes.
[qualified-browser.mjs](qualified-browser.mjs) contrôle les arguments effectifs,
utilise un pipe DevTools sans port de débogage, un profil privé neuf et nettoie
le processus/profil en succès comme en échec. Les téléchargements sont interdits,
aucun argument ne désactive le sandbox Chromium et aucun navigateur personnel
n'est utilisé. Un manque de composant provoque un échec, pas un repli.

Par défaut, la page ne peut effectuer aucune requête réseau interceptée.
Les scénarios de l'application autorisent une liste exacte de GET et POST
sur une seule origine loopback éphémère, jamais 3210, 11434 ou 6333.
Le serveur de test utilise un faux modèle et un audit temporaire.
Les sources du frontend sont Rust ; le test charge réellement son WASM.

## Audits et limites

[bundled-review.json](bundled-review.json) inventorie les composants incorporés
à Puppeteer. [audit.mjs](audit.mjs) inclut ces composants, les dépendances dev
et les empreintes des bundles ; une alerte ou un audit incomplet bloque.
Le contrôle de distributions [inspect-distributions.mjs](inspect-distributions.mjs)
reste un outil de revue, pas une installation implicite.

Le filtrage des requêtes de page n'est pas un pare-feu OS. La CSP de notre
application interdit workers/frames ; on ne prétend pas contrôler tous les
sockets de tous les processus Chromium. Les empreintes ne rendent pas les
fichiers immuables face au même utilisateur. L'empreinte de l'archive Chromium
provient du téléchargement HTTPS officiel contrôlé, pas d'une signature
Google indépendante. Aucun codec/FFmpeg supplémentaire n'est installé.

Les anciennes configurations de navigateur et sondes de composants ont été
retirées : les tests conservés qualifient l'outillage et l'application réelle.
