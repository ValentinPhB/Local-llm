# Contrat d’autorisation MCP futur

## État actuel

Aucun serveur MCP, outil, credential ou action MCP n’est installé dans ce
laboratoire. Le rôle `mcp_read_only` est donc une **intention de politique** :
il ne donne actuellement accès à rien et n'ouvre aucune connexion.

Oscar possède ce rôle via le groupe fictif `MCP_READERS`. Il possède aussi le
rôle documentaire distinct `public_welcome_reader`, limité au seul fichier
`public-welcome`. Les deux autorisations ne se mélangent pas.

## Règle pour Oscar

Lorsqu’un MCP sera ajouté et approuvé, Oscar pourra utiliser uniquement ses
actions explicitement classifiées `read`. Il ne pourra jamais utiliser une
action `write`, `create`, `update`, `delete`, `execute`, `admin` ou une action
inconnue.

```text
MCP enregistré + action exacte "read" + rôle mcp_read_only -> autorisation possible
toute autre situation                                  -> refus par défaut
```

Le mot « possible » est important : chaque nouveau MCP devra être enregistré
avec ses actions, paramètres, données accessibles et tests de refus avant son
activation. Le rôle ne crée pas un accès automatique à un outil découvert plus
tard, ni à toutes les informations que cet outil pourrait théoriquement lire.

## Contrat à imposer à chaque futur MCP

Avant connexion, son registre devra déclarer au minimum :

| Élément | Exigence pour Oscar |
| --- | --- |
| Identifiant du MCP | connu et approuvé explicitement |
| Action | exactement `read` |
| Ressource et paramètres | liste limitée et validée côté serveur |
| Credentials | identifiant dédié, sans privilège d’écriture |
| Journal d’audit | identité, MCP, action et résultat, sans secret ni contenu sensible |
| Tests | lecture autorisée, écriture/refus, MCP inconnu/refus, paramètre interdit/refus |

Cette règle permettra de réutiliser le rôle `mcp_read_only` pour les MCP futurs
approuvés, tout en conservant le principe de refus par défaut.
