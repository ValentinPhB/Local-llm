# Installation, CI/CD et livraison locale

## Responsabilités et versions

Le dépôt public GitHub est ValentinPhB/ChatPurp, branche main. Une modification
de fichier, un succès local et un succès GitHub Actions sont trois états distincts.
La migration n’est pas déclarée livrée avant la bascule et les contrôles distants.

| Brique | Installation / référence |
| --- | --- |
| Outils Apple | Command Line Tools et SDK macOS, vérifiés par xcode-select -p et clang --version. |
| Rust / Cargo | 1.98.1 via rustup 1.29.1 ; [installateur local](../tools/rust/install.sh), empreinte avant exécution, aucun profil shell modifié. |
| Dépendances Rust | Cargo.toml et Cargo.lock, versions exactes ; compilation ARM64 et WASM. |
| Transformation WASM | wasm-bindgen-cli-support 0.2.128 dans [tools/wasm-build](../tools/wasm-build/Cargo.toml) ; pas de CLI complète, pas de Bun. |
| Node / navigateur | [runtime-lock.json](../tools/browser-tests/runtime-lock.json) fixe archives et exécutables ; [installateur](../tools/browser-tests/install-runtime.mjs). |
| Test navigateur | Puppeteer Core 25.10.0, package-lock ; installation sans scripts npm. |
| SpecDD | CLI 1.1.1 ; [guide dédié](../tools/specdd/README.md). |
| Ollama | Version locale vérifiée 0.33.3 ; binaire natif macOS, indépendant du build Rust. |
| Modèles | qwen3:4b et embeddinggemma téléchargés par Ollama, jamais dans Git. |
| Qdrant | Image v1.19.1-unprivileged épinglée par digest, volume nommé dédié. |

## Installer le projet après clonage

Prérequis : Git, outils Apple, curl, tar/unzip, jq et un Node de bootstrap ≥ 22.
Le Node de bootstrap sert uniquement à installer les archives verrouillées ;
les tests utilisent ensuite le Node dédié vérifié par empreinte.

~~~sh
sh tools/rust/install.sh
node tools/browser-tests/install-runtime.mjs
export PATH="$PWD/.local/browser-runtime/node-v22.23.2-darwin-arm64/bin:$PATH"
npm --prefix tools/specdd ci --omit=dev --ignore-scripts --no-fund --no-audit
npm --prefix tools/browser-tests ci --ignore-scripts --no-fund --no-audit
sh tools/rust/run.sh cargo fetch --locked
sh tools/rust/run.sh cargo fetch --manifest-path tools/wasm-build/Cargo.toml --locked
node tools/rust/audit-workspace.mjs
sh tools/browser-tests/run.sh audit
npm --prefix tools/specdd run audit
~~~

Le PATH ci-dessus ne concerne que le terminal courant. Les installations sont
dans .local et les node_modules des outils. Aucun composant applicatif n’est
installé globalement. Les commandes fetch et audit ont besoin du réseau ;
les builds et tests déterministes suivants utilisent --locked --offline.

Avant compilation Dioxus, preflight.sh vérifie les JS préconstruits avec cinq
tests du mécanisme de cache amont. Ce contrôle n’est pas une signature de sécurité.
Les checksums Cargo et les audits de dépendances restent nécessaires.

## Installer / démarrer Ollama sans modifier le lab existant

Ollama est déjà installé sur la machine de développement : ne pas réinstaller
ni démarrer une seconde instance sur 11434. Pour une machine neuve, prendre
l’archive native de la [release officielle 0.33.3](https://github.com/ollama/ollama/releases/tag/v0.33.3).
Archive CLI macOS : ollama-darwin.tgz ; SHA-256 publié :
342db03df80bb9db84ff64246031bd5f70c09b59ff52fa5cc9aaae3476cc4a9d.
Télécharger dans un dossier temporaire dédié, comparer cette empreinte avant
extraction, puis conserver l’exécutable et ses bibliothèques ensemble.
L’application macOS alternative est Ollama-darwin.zip, SHA-256 :
335f1a11299f5f60dc2d5f2651cf12af9d3c303812c68e978be3e45ea7d6eaf4.

Démarrage CLI d’une instance arrêtée, avec le binaire installé accessible :

~~~sh
OLLAMA_HOST=127.0.0.1:11434 OLLAMA_NO_CLOUD=1 ollama serve
~~~

Ces variables imposent loopback et désactivation du cloud. L’application macOS
utilise ses réglages propres ; le modèle de configuration
[server.json.example](../config/ollama/server.json.example) désactive aussi le cloud.
Vérifier la configuration effective après redémarrage, selon la
[documentation officielle](https://docs.ollama.com/faq).
Ne pas écraser une configuration personnelle existante.

Dans un autre terminal, téléchargement explicite des modèles sur machine neuve :

~~~sh
ollama pull qwen3:4b
ollama pull embeddinggemma
ollama list
curl -fsS http://127.0.0.1:11434/api/version
~~~

Un tag de modèle peut évoluer : enregistrer le digest effectif et les licences
lors d’une mise à jour. Le tag seul n’offre pas la reproductibilité d’un
Cargo.lock. La migration Rust ne retélécharge ni ne réentraîne ces modèles.

## Installer / démarrer Qdrant sur une machine neuve

Docker Desktop doit fonctionner. L’instance du lab utilise l’utilisateur
1000:1000, 1 Gio de RAM, un volume nommé llm-lab-qdrant-data, et le seul port
hôte 127.0.0.1:6333. Aucun montage du répertoire personnel ni socket Docker.

~~~sh
docker volume create llm-lab-qdrant-data
docker run -d --name llm-lab-qdrant --memory=1g --user 1000:1000 -p 127.0.0.1:6333:6333 -v llm-lab-qdrant-data:/qdrant/storage qdrant/qdrant:v1.19.1-unprivileged@sha256:801777072776dc81b2a9dd2007b2ed487571f21ecd30efffd15ddb1671f2193d
curl -fsS http://127.0.0.1:6333/readyz
~~~

Ces commandes de création ne sont pas à rejouer sur un conteneur existant :
docker start llm-lab-qdrant suffit s’il est arrêté. Le volume ne doit pas être
supprimé lors d’une livraison de l’API. L’indexation reste une commande
administrative explicite, décrite dans [recherche et indexation](semantic-rag-design.md).

## Construire, vérifier et lancer Rust

~~~sh
sh tools/check.sh
sh tools/qdrant-test.sh
sh tools/rust/build-app.sh
~~~

check.sh regroupe formatage, Clippy natif/WASM sans avertissement, tests métier
et API, SpecDD, hygiène/liens, tests du transformateur et navigateur réel.
qdrant-test.sh crée un conteneur séparé, sans volume du lab, avec 256 Mio,
un port aléatoire loopback, puis le supprime même en cas d’erreur.
Aucun test de ce contrôle ne doit viser le port 6333 du lab.

build-app.sh produit les binaires et affiche CHATPURP_WEB_ASSETS=<dossier>.
Démarrer avec ce dossier exact, après vérification qu'aucune instance n'écoute déjà :

~~~sh
.local/rust/target/aarch64-apple-darwin/debug/chatpurp-api "$PWD" "<dossier CHATPURP_WEB_ASSETS affiché>"
~~~

L’adresse est http://127.0.0.1:3211. Une configuration invalide empêche l’écoute.
Pas de service permanent installé ; arrêter le processus dédié avec Ctrl-C
depuis son terminal. Ne pas utiliser de kill global sur Python, Rust ou Ollama.

## CI et CD

[Workflow](../.github/workflows/tests.yml) exécuté sur push/PR vers main,
permissions GitHub en lecture minimale :

Les actions sont verrouillées par commit : checkout 7.0.1, setup-node 7.0.0,
Gitleaks Action 3.0.0. Leurs moteurs Node 24 sont distincts du Node 22.23.2
utilisé pour nos tests. Identifiants Git non persistés et cache npm automatique
désactivé. Références officielles : [checkout](https://github.com/actions/checkout/releases/tag/v7.0.1),
[setup-node](https://github.com/actions/setup-node/releases/tag/v7.0.0),
[Gitleaks](https://github.com/gitleaks/gitleaks-action/releases/tag/v3.0.0).

- Contrats SpecDD : installation verrouillée, audit, lint, tests.
- Rust sur macOS 26 ARM64 : installations vérifiées, audit des dépendances,
  build natif/WASM, tests HTTP/filesystem puis vrais parcours Chromium.
- Contrat Qdrant sur Ubuntu : binaire Rust contre un Qdrant éphémère sur 6334,
  sans modèle ni donnée du lab.
- Détection de secrets : Gitleaks.

Le choix du runner ARM64 est confirmé par la
[référence GitHub](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).
Il ne remplace pas l’exécution effective du workflow.
Les tests n’appellent pas un LLM distant ni l’Ollama personnel. Le faux modèle
rend le parcours CI déterministe ; il ne mesure pas la qualité de génération.

CD actuelle : livraison locale contrôlée, pas de déploiement automatique
Internet ni de publication de crates. Une release doit identifier commit,
Cargo.lock, toolchain, assets du même build, modèles effectifs et résultats CI.
Une installation réussie ne constitue pas un audit exhaustif des fournisseurs.

## Bascule, nettoyage et retour arrière

La bascule approuvée remplace Python par Rust sur 3211. Le code Python et son
job CI sont retirés ; aucun service applicatif n'est attendu sur 3210.
Le cookie Rust et l'audit restent distincts des anciens : un port différent
ne suffit pas à isoler les cookies.

Après chaque démarrage, contrôle automatisé d'intégration locale explicite :

~~~sh
.local/browser-runtime/node-v22.23.2-darwin-arm64/bin/node tools/live-smoke.mjs
~~~

Ce contrôle utilise Oscar et deux prompts fictifs fixes, vérifie santé,
session, autorisation/refus, récupération, réponses du vrai Ollama et déconnexion.
Il ne stocke aucun jeton ni réponse, ne modifie pas Qdrant, et n'est pas lancé
par la CI : sa disponibilité dépend des services et ses réponses ne sont pas
déterministes. Il ne remplace pas la suite de contrats à fournisseurs fictifs.

Publier le dépôt sans archives ni documents obsolètes après les vérifications.
Ne pas effacer modèles, volume Qdrant ou données locales pour « nettoyer Git ».

Un retour arrière utilise un commit publié antérieur dans un dossier distinct
et ses commandes de lancement ; ne pas réinitialiser brutalement le worktree.
Pour une future mise à jour Rust, conserver le binaire ET ses assets cohérents.
Tout redémarrage ferme les sessions. Le remplacement de l’index sémantique
n’est pas transactionnel et nécessite une procédure indépendante.

## Rapport destiné à OneNote

~~~sh
.local/browser-runtime/node-v22.23.2-darwin-arm64/bin/node tools/export-onenote.mjs
~~~

Cette commande génère .local/reports/chatpurp/rapport.html et deux schémas PNG,
à partir des guides courants, sans accès réseau du navigateur ni connexion à
OneNote. Le HTML est autonome ; les PNG peuvent être insérés séparément.
Le transfert et sa vérification dans le OneNote de l’utilisateur restent à
faire après ouverture de l’accès et choix de la destination.
