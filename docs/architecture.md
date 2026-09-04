# Architecture cible

## Vue d'ensemble

```text
Utilisateur
    |
    v
Open WebUI
  - authentification
  - utilisateurs, groupes
  - RBAC / ACL
  - recherche RAG
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
| Open WebUI | Identifier l'utilisateur, appliquer les permissions et préparer le contexte | Déléguer une décision d'autorisation au LLM |
| RAG | Rechercher parmi les documents déjà autorisés et retourner les passages pertinents | Rendre un document non autorisé accessible au modèle |
| Ollama | Exécuter localement l'inférence et servir son API | Gérer les droits applicatifs ou exposer l'API hors de la machine |
| LLM | Produire une réponse à partir du prompt et du contexte reçus | Accéder directement au filesystem, décider des permissions ou utiliser des credentials d'administration |

## Flux RAG attendu

```text
Question de l'utilisateur
    -> authentification
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

- Ollama et Open WebUI devront écouter sur `localhost` uniquement.
- Le LLM ne recevra aucun montage direct du filesystem du Mac.
- Les documents de démonstration seront fictifs et importés explicitement dans la base de connaissance.
- Aucun MCP, outil externe ou credential n'est prévu à ce stade.
