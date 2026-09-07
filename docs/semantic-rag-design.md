# RAG sémantique contrôlé — conception approuvée

## État et objectif

Le RAG actuellement actif est lexical : il compte les mots communs entre une
question et les documents déjà autorisés. Cette conception décrit son évolution
vers une recherche sémantique locale, capable de rapprocher des formulations de
sens voisin sans confier les droits d'accès au modèle.

Le choix approuvé est `embeddinggemma` dans Ollama et Qdrant comme base
vectorielle locale. `embeddinggemma` est installé localement et son endpoint
Ollama a produit un vecteur de contrôle pour une phrase fictive. Qdrant est
démarré mais reste vide. Les adaptateurs `semantic_retrieval/clients.py` et
l'indexeur contrôlé `semantic_retrieval/indexer.py` et le writer Qdrant existent
et sont testés avec des services simulés ; aucune route API ne les appelle
encore. Aucun document n'est indexé. La récupération lexicale et le chat RAG
existants restent donc la référence active.

## Composants retenus

| Composant | Responsabilité | Frontière |
| --- | --- | --- |
| Ollama `embeddinggemma` | Produit les vecteurs des passages et questions via `/api/embed`. | `127.0.0.1:11434`, jamais appelé par le navigateur. |
| Qdrant `qdrant/qdrant:v1.19.1-unprivileged` | Stocke les vecteurs et le texte des passages fictifs ; recherche les plus proches. | Conteneur ARM64, port REST publié seulement sur `127.0.0.1:6333`. |
| API Python | Applique session, RBAC/ACL, filtre Qdrant, bornage, audit et relais LLM. | Seul composant qui parle à Qdrant ou Ollama. |

`embeddinggemma` occupe environ 622 Mo sur disque et convient à la recherche
multilingue locale. Qdrant tourne sous l'utilisateur `1000:1000`, avec un
volume Docker nommé dédié, jamais le répertoire personnel, la racine du système
ou le socket Docker. Il est limité à 1 Go de RAM et publié seulement sur
`127.0.0.1:6333`. Avec les quinze documents fictifs, les vecteurs occuperont un
espace négligeable face au modèle.

## Flux cible, non négociable

```text
Indexation contrôlée, déclenchée localement
document fictif déclaré dans la politique
-> découpage en passages bornés
-> embedding local de chaque passage
-> Qdrant : vecteur + resource_id + classification + passage

Question sémantique
-> session vérifiée
-> groupes -> rôles -> liste des resource_id ACL autorisés
-> embedding local de la question
-> Qdrant avec filtre serveur resource_id IN [liste autorisée]
-> au plus trois passages autorisés
-> API ou contexte RAG Ollama
```

Le navigateur ne fournit jamais le filtre, un vecteur, un `resource_id`, une
source ou un passage. Qdrant ne constitue pas une source d'autorisation : son
filtre est construit par l'API après ACL. Le LLM ne reçoit aucun vecteur, accès
Qdrant, chemin local ou document hors résultat filtré.

## Indexeur contrôlé déjà présent, non encore connecté

`ControlledIndexer` ne prend que des `resource_id` déclarés dans la politique,
jamais des chemins envoyés par un utilisateur. Il relit chaque document avec
`read_policy_document`, qui contrôle le répertoire, la taille et les métadonnées
de front matter. Il retire ensuite ces métadonnées du texte recherché, découpe
le corps en paragraphes puis en phrases, et limite un passage à 700 caractères.
Quand un passage suivant ne tient plus, la dernière phrase du précédent est
réutilisée si elle fait au plus 160 caractères : le contexte reste lisible sans
dépasser la limite. Une phrase exceptionnellement trop longue est coupée au
dernier espace possible.

Le fournisseur d'embeddings et le writer sont injectés. L'indexeur construit
donc d'abord un lot complet et cohérent (vecteurs finis, mêmes dimensions), puis
le remet une seule fois au writer. Pour l'instant, les tests injectent seulement
un faux fournisseur et un faux writer : aucune écriture Qdrant ne peut être
déclenchée par ce code. L'ACL de chaque utilisateur est toujours appliquée plus
tard, lors de la requête de recherche ; l'indexation est un travail
administratif local sur les documents explicitement déclarés.

Le writer `QdrantIndexWriter` cible exclusivement la collection fixe
`lab_semantic_documents` sur `127.0.0.1:6333`. Il valide le lot avant tout
appel, rejette les classifications, dimensions et chunks invalides, puis
reconstruit cette seule collection avec des identifiants de points déterministes.
Cette reconstruction est par nature destructive pour **cette collection de
démonstration** ; elle ne sera jamais appelée par une route utilisateur. Pour
l'instant, elle est testée avec un faux Qdrant et aucun code ne l'instancie pour
écrire dans le conteneur réel.

## Données indexées et rétention

Chaque point Qdrant contiendra uniquement : `resource_id`, `classification`,
`chunk_id`, texte du passage et vecteur. Les quinze documents sont fictifs,
mais ce même contenu deviendrait sensible avec de vraies données : la base doit
donc rester locale et son volume Docker ne sera pas versionné dans Git.

L'indexation reconstruira la collection de démonstration de manière
déterministe. Un changement de document, de découpage ou de modèle impose une
réindexation complète ; mélanger les vecteurs issus de modèles différents est
interdit.

## Déploiement progressif et repli

La première version introduira une route sémantique distincte, sans modifier
`POST /api/retrieve` ni `POST /api/rag-chat`. Elle permettra de comparer les
résultats sans changement silencieux du comportement actuel. Une indisponibilité
d'Ollama embeddings ou de Qdrant renverra `503` : aucun repli implicite vers la
recherche lexicale, aucun document et aucun message ne seront envoyés au LLM.

Une évolution ultérieure, explicitement décidée, pourra faire utiliser le
rétrieval sémantique par le RAG. Elle mettra alors à jour le flux de requête.

## État de la couche cliente

`semantic_retrieval/clients.py` fixe les deux seules destinations réseau :
`127.0.0.1:11434/api/embed` pour Ollama et `127.0.0.1:6333` pour Qdrant. Il
valide les vecteurs, interdit une liste ACL vide de devenir une recherche globale
et rejette une ressource renvoyée par Qdrant qui ne figure pas dans le filtre
autorisé. `QdrantIndexWriter` ajoute l'écriture administrative isolée : son nom
de collection est imposé, l'URL reste loopback et il n'est pas importé par
`ui/server.py`.

## Contrôles automatisés obligatoires

Avant activation, la CI doit démontrer automatiquement :

1. un faux fournisseur d'embeddings déterministe permet des tests sans Ollama ;
2. l'indexeur refuse une politique ACL invalide ou un identifiant hors politique
   avant tout embedding ou toute écriture ;
3. le découpage conserve les fins de phrase, respecte la taille maximale et
   produit un chevauchement borné ;
4. les requêtes Qdrant reçoivent exactement le filtre `resource_id` construit
   depuis l'ACL, en particulier Oscar -> `public-welcome` seulement ;
5. une source RH ou IT interdite n'est ni renvoyée ni transmise au faux LLM ;
6. l'indisponibilité du fournisseur d'embeddings ou de Qdrant bloque la route
   avant toute lecture ou appel LLM ;
7. question, vecteurs, passages et résultats ne sont jamais inscrits dans le
   journal d'audit ;
8. une intégration Qdrant réelle s'exécute automatiquement dans GitHub Actions
   avec un conteneur de service ; aucune recette manuelle ne constitue une
   validation.

## Références techniques

- [Ollama — Generate embeddings](https://docs.ollama.com/api/embed)
- [Ollama — EmbeddingGemma](https://ollama.com/library/embeddinggemma)
- [Qdrant — filtres de recherche](https://qdrant.tech/documentation/search/filtering/)
