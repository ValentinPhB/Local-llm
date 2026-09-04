# Architecture cible

## Vue d'ensemble

```text
Utilisateur
    |
    v
Chatbox (client local natif)
  - interface de conversation
  - conversations locales
  - aucune fonction d'agent, RAG ou MCP configurée
    |
    | prompt + passages autorisés seulement
    v
Ollama (API locale)
    |
    v
Modèle local sur Apple Silicon
```

## Responsabilités et frontières

| Couche | Responsabilité | Ne doit pas faire |
| --- | --- | --- |
| Chatbox | Présenter l'interface et transmettre les messages à l'API locale | Gérer des droits, choisir des documents ou fournir des outils au modèle |
| Future couche RAG | Rechercher parmi les documents déjà autorisés et retourner les passages pertinents | Rendre un document non autorisé accessible au modèle |
| Ollama | Exécuter localement l'inférence et servir son API | Gérer les droits applicatifs ou exposer l'API hors de la machine |
| LLM | Produire une réponse à partir du prompt et du contexte reçus | Accéder directement au filesystem, décider des permissions ou utiliser des credentials d'administration |

## Flux RAG attendu

```text
Question de l'utilisateur
    -> authentification (future couche dédiée)
    -> RBAC / ACL déterministes
    -> recherche dans les sources autorisées
    -> passages autorisés uniquement
    -> contexte envoyé au LLM
    -> réponse
```

Les instructions d'un utilisateur ou d'un document ne modifient jamais les
permissions. Un prompt injection peut influencer le texte généré, mais ne doit
pas permettre de contourner le filtre d'autorisation technique.

## Limites initiales

- Ollama écoute sur `localhost` uniquement. Chatbox est un client natif, pas un service web exposé.
- Aucun fournisseur cloud n'est configuré dans Chatbox ; il sera relié uniquement à l'API locale d'Ollama.
- Le LLM ne recevra aucun montage direct du filesystem du Mac.
- Les documents de démonstration seront fictifs et importés explicitement dans la base de connaissance.
- Aucun MCP, outil externe ou credential n'est prévu à ce stade.
