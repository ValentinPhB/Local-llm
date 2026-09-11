# Recherche, RAG et indexeur contrôlé

## État

Le RAG raccordé à l’API Rust est lexical. Les capacités sémantiques préparées
sont également portées en Rust, mais ni indexation du lab ni route sémantique
n’ont été activées. Qdrant et embeddinggemma restent des dépendances locales
préparées. Les tests utilisent des doubles et un Qdrant éphémère isolé.

## Récupération lexicale

Après session valide et audit, le core calcule la liste des ressources
autorisées, puis lit seulement ces fichiers. Le front matter ne participe pas
au classement. Les mots Unicode alphanumériques ou avec underscore, d’au moins
deux caractères, sont mis en minuscules ; score = nombre de termes distincts
communs. Tri par score décroissant puis resource_id croissant.
Au plus trois extraits, espaces normalisés, 500 caractères chacun.

Une requête de recherche contient 1 à 500 caractères après trim.
Le chat RAG accepte le message jusqu’à 8000 caractères et ne lui applique pas
la limite de la route de recherche. Le contexte serveur cite les ressources
et présente leurs passages comme des données, jamais des instructions.
Le navigateur ne peut imposer ni contexte, ni sources, ni modèle, ni destination.

## Embeddings et recherche vectorielle préparés

Un embedding représente un texte par une liste de nombres ; la dimension
est produite par le modèle, pas par Qdrant. L’adaptateur impose embeddinggemma,
le port 11434 et /api/embed : texte ≤ 8000 caractères, délai 30 s, réponse
≤ 256 KiB. Un seul vecteur est attendu, de 1 à 4096 coordonnées finies.

La recherche Qdrant utilise le port 6333, la collection lab_semantic_documents
et /points/query. Le filtre obligatoire est resource_id parmi les ressources
autorisées côté serveur. Une liste vide produit zéro appel réseau.
Délai 5 s, réponse ≤ 256 KiB, au plus trois résultats ; le payload est revérifié
contre l’identifiant ET la classification de la politique. Extraits ≤ 500 caractères.
Un retour hors ACL ou mal formé invalide toute la réponse.

## Découpage et indexation administrative

Le binaire chatpurp-index est distinct du serveur HTTP. Il prend des identifiants
déclarés, pas des chemins client. La préparation valide toute la liste avant
lecture, utilise le lecteur contrôlé et retire les métadonnées.
Découpage paragraphes/phrases, maximum 700 caractères ; dernière unité
réutilisée si ≤ 160 caractères ET si elle tient avec l’unité suivante.
Une phrase exceptionnellement longue est coupée au dernier espace disponible,
sinon à une frontière de caractère Unicode.

Ce découpage est déterministe, pas une compréhension linguistique parfaite :
abréviations, tableaux et titres peuvent justifier un futur découpeur spécialisé.
Les tests contrôlent les bornes, le recouvrement et les mots Unicode longs.

Le lot complet est préparé puis vectorisé ; 512 passages maximum, dimensions
homogènes, coordonnées finies, classifications PUBLIC/RH/IT, couples
(resource_id, chunk_id) uniques. Aucune écriture Qdrant avant validation du lot.
Les identifiants de points sont des UUID v5 déterministes, compatibles avec
le namespace initial. SHA-1 sert ici à l’identification, pas à signer une donnée.

## Deux modes explicites

~~~sh
# Lecture/plan seulement : aucun embedding, aucune écriture réseau.
.local/rust/target/aarch64-apple-darwin/debug/chatpurp-index --plan "$PWD" public-welcome
~~~

Le mode --replace-lab-index est administratif et destructif pour la seule
collection lab_semantic_documents : suppression (404 admis si déjà absente),
création Cosine avec la dimension validée, insertion du lot avec wait=true.
Il n’a pas été exécuté contre le lab. Une panne entre ces opérations peut
laisser la collection absente ou partielle : il n’y a pas de remplacement
transactionnel ni de retour arrière automatique.

Les passages et vecteurs sont des données dérivées, pas une source de droits.
Avant un futur raccordement : décision explicite, index de test contrôlé,
tests d’accès complets sur la route et stratégie de réindexation/révocation.
Aucun MCP ne fait partie de cette étape.
