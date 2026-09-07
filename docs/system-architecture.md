# Architecture du système

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
  - récupération lexicale après ACL
  - chat RAG : contexte construit côté serveur
  - couche sémantique préparée, mais non activée
    |
    | message seul ou message + extraits autorisés
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
| Lecteur documentaire | Lire un fichier explicitement autorisé et déclaré par la politique | Accepter un chemin client ou lire avant l'ACL |
| Récupération lexicale | Classer des extraits parmi les documents déjà autorisés | Lire ou classer un document non autorisé |
| Génération augmentée active | Ajouter au prompt uniquement des extraits récupérés et autorisés | Accepter un contexte fourni par le navigateur ou un extrait non filtré |
| Couche sémantique préparée | Préparer embeddings, filtre Qdrant et indexation par politique | Écrire dans Qdrant ou servir des requêtes avant activation explicite |
| Future passerelle MCP | Autoriser une action déclarée pour un MCP approuvé | Donner un accès implicite à un MCP ou à une action inconnus |
| Ollama | Exécuter localement l'inférence et servir son API | Gérer les droits applicatifs ou exposer l'API hors de la machine |
| LLM | Produire une réponse à partir du prompt et du contexte reçus | Accéder directement au filesystem, décider des permissions ou utiliser des credentials d'administration |

## Flux RAG actif

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

La conception détaillée de cette frontière est conservée dans
[`authorization-security-boundary.md`](authorization-security-boundary.md). Le
contrat de la simulation SSO est dans
[`demo-sso-authentication.md`](demo-sso-authentication.md). La récupération
lexicale active est définie dans
[`lexical-rag-retrieval.md`](lexical-rag-retrieval.md).

## Limites initiales

- L'interface et l'API du lab écoutent sur `127.0.0.1:3210`; Ollama écoute sur
  `127.0.0.1:11434` et Qdrant sur `127.0.0.1:6333`.
- Le navigateur parle à l'API du lab, jamais directement à Ollama ou Qdrant.
- Le LLM ne recevra aucun montage direct du filesystem du Mac.
- Quinze documents de démonstration fictifs sont versionnés dans le dépôt,
  classifiés et référencés par une ACL ; ils sont lus seulement après ACL et
  peuvent être transmis à Ollama uniquement comme extraits lexicaux autorisés
  par `POST /api/rag-chat`.
- Qdrant est vide : aucun index vectoriel persistant n'est encore alimenté.
- Aucun MCP, outil externe ou credential n'est prévu à ce stade.
