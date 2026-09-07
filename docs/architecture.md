# Architecture cible

## Vue d'ensemble

```text
Utilisateur
    |
    v
Interface locale minimale du projet
  - choix explicite d'une identité fictive (mode simulation)
  - cookie de session signé et court
    |
    v
API locale du laboratoire
  - vérification du jeton et traduction groupes -> rôles
  - RBAC/ACL sur 15 documents fictifs classifiés
  - lecteur documentaire après ACL uniquement
  - aucune fonction RAG ou MCP configurée
    |
    | prompt seul ; passages autorisés plus tard
    v
Ollama (API locale)
    |
    v
Modèle local sur Apple Silicon
```

## Responsabilités et frontières

| Couche | Responsabilité | Ne doit pas faire |
| --- | --- | --- |
| Interface locale | Présenter l'interface et transmettre un message à l'API locale | Gérer des droits, choisir des documents ou fournir des outils au modèle |
| Simulation SSO locale | Émettre un jeton signé pour une identité fictive choisie explicitement | Prouver une identité réelle ou remplacer un IdP d'entreprise |
| API du laboratoire | Vérifier la session, traduire les groupes en rôles et appliquer l'ACL | Croire une identité libre envoyée avec un chat, ou déléguer un droit au LLM |
| Lecteur documentaire | Lire un fichier explicitement autorisé et déclaré par la politique | Accepter un chemin client, lire avant l'ACL ou transmettre le fichier au LLM |
| Future couche RAG | Rechercher parmi les documents déjà autorisés et retourner les passages pertinents | Rendre un document non autorisé accessible au modèle |
| Future passerelle MCP | Autoriser une action déclarée pour un MCP approuvé | Donner un accès implicite à un MCP ou à une action inconnus |
| Ollama | Exécuter localement l'inférence et servir son API | Gérer les droits applicatifs ou exposer l'API hors de la machine |
| LLM | Produire une réponse à partir du prompt et du contexte reçus | Accéder directement au filesystem, décider des permissions ou utiliser des credentials d'administration |

## Flux RAG attendu

```text
Question de l'utilisateur
    -> jeton d'identité vérifié (simulation locale aujourd'hui, OIDC demain)
    -> RBAC / ACL déterministes
    -> recherche dans les sources autorisées
    -> passages autorisés uniquement
    -> contexte envoyé au LLM
    -> réponse
```

Les instructions d'un utilisateur ou d'un document ne modifient jamais les
permissions. Un prompt injection peut influencer le texte généré, mais ne doit
pas permettre de contourner le filtre d'autorisation technique.

La conception détaillée de cette frontière avant RAG et MCP est conservée dans
[`authorization-boundary.md`](authorization-boundary.md). Le contrat de la
simulation SSO est dans [`demo-sso.md`](demo-sso.md).

## Limites initiales

- Ollama et l'interface locale écoutent sur `localhost` uniquement.
- L'interface transmet uniquement vers `http://127.0.0.1:11434` et ne configure aucun fournisseur cloud.
- Le LLM ne recevra aucun montage direct du filesystem du Mac.
- Quinze documents de démonstration fictifs sont versionnés dans le dépôt,
  classifiés et référencés par une ACL ; ils sont lus seulement après ACL, ne
  sont pas indexés et ne sont jamais transmis au modèle.
- Aucun MCP, outil externe ou credential n'est prévu à ce stade.
