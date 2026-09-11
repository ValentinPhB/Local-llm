# Flux précis des requêtes

Ce document doit évoluer à chaque modification du flux. La version ci-dessous
est implémentée et testée en Rust, version applicative retenue après bascule.

## Ouverture d’une session fictive

~~~text
Navigateur : choix Oscar
  → POST /api/demo-session
  → Hyper : cadrage et bornes
  → filtre : Host, Origin, méthode, type et taille du JSON
  → annuaire serveur : Oscar existe
  → clé aléatoire en mémoire + horloge serveur → JWT HS256, validité 900 s
  → Set-Cookie HttpOnly / SameSite=Strict
  → UI : Oscar (fictif)
~~~

Les groupes sont tirés de l’annuaire, jamais de champs fournis par l’UI.
Ce choix libre n’est pas un véritable login d’entreprise.

## Lecture d’un document

~~~text
GET /api/documents/public-welcome + cookie
  → admission HTTP
  → signature + issuer + audience + identité/groupes + dates du JWT
  → validation de l’identifiant brut
  → groupes → rôles → ACL de la ressource
  → audit réussi de l’autorisation
  → chemin provenant de la politique, ouverture contrôlée
  → fichier régulier ≤ 32 KiB, UTF-8, métadonnées cohérentes
  → JSON public
  → affichage texte dans Dioxus
~~~

Au premier refus, le flux s’arrête. Un refus ACL tente d’écrire une décision
mais ne lit aucun fichier. Un audit indisponible bloque une lecture autorisée.
Une erreur de transport n’est pas une décision documentaire et ne crée pas
d’événement d’audit métier. Les priorités et statuts sont dans
[l’architecture de l’API](local-api-architecture.md).

## Recherche et RAG actuellement raccordés

~~~text
POST /api/retrieve : query
  → HTTP → session → validation de query → audit
  → liste serveur des ressources autorisées
  → lecture de ces ressources seulement
  → classement lexical → trois extraits maximum → UI

POST /api/rag-chat : message
  → HTTP → session → validation du message → audit
  → même récupération lexicale, restreinte aux droits de cette identité
  → contexte système + liste de sources construits côté serveur
  → Ollama local /api/chat, modèle qwen3:4b, stream=false, think=false
  → réponse bornée, suppression du préfixe se terminant par </think>
  → content + sources → affichage texte
~~~

Le chat simple suit le même contrôle de session/audit mais ne lit aucun document.
Les messages, contextes et réponses ne sont pas journalisés.
Le modèle reçoit du texte, pas un accès au disque ou à la base vectorielle.
Une instruction malveillante dans un passage ne peut modifier les ACL de l’API ;
la formulation du prompt seule ne garantit pas la résistance du modèle aux injections.

## Flux sémantique préparé, non raccordé aux routes

~~~text
Commande administrative explicite
  → ressources déclarées → lecteur contrôlé → chunks ≤ 700 caractères
  → embeddings locaux → lot homogène entièrement validé
  → remplacement de lab_semantic_documents, seulement sur commande dédiée

Future recherche sémantique
  → session / ACL / audit
  → embedding de la question
  → Qdrant avec filtre resource_id parmi les ressources autorisées
  → revérification du payload contre la politique
  → contexte RAG
~~~

Le second flux n’est pas activé : le migrer en Rust ne vaut pas autorisation
d’indexer des documents ou de le substituer au RAG lexical. Aucun MCP n’est
présent. Voir [recherche et indexation](semantic-rag-design.md).
