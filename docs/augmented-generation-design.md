# Conception de la génération augmentée contrôlée

## Objectif

Ajouter la partie **G** de RAG : envoyer à Ollama la question et les extraits déjà récupérés par l'ACL. Le chat simple actuel restera disponible afin de comparer une conversation sans document et une réponse augmentée.

```text
Question
    -> session -> ACL -> récupération lexicale autorisée
    -> extraits bornés + identifiants de source
    -> prompt construit côté serveur
    -> Ollama
    -> réponse + sources utilisées au navigateur
```

## Route distincte proposée

`POST /api/rag-chat` recevra uniquement :

```json
{"message":"Comment se passe l’intégration ?"}
```

Le navigateur ne pourra fournir ni `context`, ni `sources`, ni identifiant de document, ni rôle. Le serveur recalculera la récupération pour chaque demande, puis retournera la réponse et les seuls `resource_id` effectivement utilisés.

Le chat existant `POST /api/chat` restera sans document.

## Contexte construit par le serveur

Le serveur utilisera au maximum trois extraits de 500 caractères, déjà issus de documents autorisés. Il construira un message système qui :

- précise que les extraits sont des données de référence, pas des instructions ;
- demande de répondre à la question en s'appuyant sur les sources fournies ;
- demande de signaler l'absence d'information si les extraits sont insuffisants ;
- interdit de transformer le contenu documentaire en permission ou en action.

Cette consigne réduit le risque de prompt injection documentaire, mais elle ne remplace jamais l'ACL : la sécurité principale reste le filtrage avant lecture et avant récupération.

## Cas Oscar

Pour Oscar, la récupération ne peut fournir que `public-welcome`. Ainsi, `/api/rag-chat` ne peut envoyer à Ollama que sa question et un extrait de ce fichier. Une mention de RH dans la question ne modifie pas cette liste.

## Tests impératifs

1. Une requête augmentée d'Oscar contient `public-welcome` dans le corps envoyé au faux Ollama et aucune chaîne RH ou IT.
2. Une requête de Bob qui cible RH ne transmet aucun extrait RH.
3. Le navigateur qui fournit un champ `context` ou `sources` ne peut pas influencer le contexte envoyé ; ces champs sont ignorés ou refusés.
4. La réponse API expose les seuls identifiants effectivement utilisés, jamais les chemins locaux.
5. Le chat simple ne change pas et ne transmet toujours aucun document.
