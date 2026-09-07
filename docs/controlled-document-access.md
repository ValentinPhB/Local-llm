# Accès documentaire contrôlé

## État actuel

L'API locale peut désormais retourner le contenu d'un document fictif, mais
seulement après une décision RBAC/ACL autorisée. Ce lecteur ne fait ni recherche
plein texte, ni embeddings, ni chunking, ni appel à Ollama : ce n'est pas un
RAG.

```text
GET /api/documents/<resource_id>
    -> session signée vérifiée
    -> groupes -> rôles
    -> ACL : autoriser ou refuser
    -> seulement si autorisé : chemin déclaré dans la politique
    -> confinement dans demo-documents/ + taille + métadonnées vérifiées
    -> contenu retourné au navigateur
```

Exemple : Oscar peut appeler `GET /api/documents/public-welcome`, mais reçoit
`403` pour `GET /api/documents/public-glossary`. Dans ce second cas, aucun
fichier n'est ouvert.

## Contrat d’entrée et de sortie

La route reçoit uniquement un identifiant, par exemple `public-welcome`. Elle
ne reçoit jamais de chemin fourni par le navigateur. Un identifiant contenant
un slash, `..` ou tout caractère hors de `[a-z0-9-]` est refusé avec `400`.

Une réponse autorisée (`200`) contient :

```json
{
  "resource_id": "public-welcome",
  "classification": "PUBLIC",
  "content": "…"
}
```

Une session absente renvoie `401`. Une ressource interdite ou inconnue renvoie
`403`, sans révéler si elle existe. Un fichier ou des métadonnées incohérents
provoquent un refus interne générique (`500`) ; le contenu n'est pas retourné.

## Protections du lecteur

- L'ACL est évaluée avant l'appel à `read_policy_document`.
- Seul le chemin versionné dans `demo-policy.json` peut être utilisé.
- Le chemin résolu doit rester dans `demo-documents/`, ce qui bloque les
  traversées de répertoire et les liens symboliques sortants.
- Le fichier doit être régulier et faire au plus 32 KiB.
- Son front matter doit reproduire l'identifiant et la classification annoncés
  par la politique.
- L'API ne journalise pas son contenu et envoie `Cache-Control: no-store`.

## Limite volontaire

Cette route de lecture directe retourne un document autorisé au navigateur ;
elle n'appelle jamais Ollama. Le même lecteur est réutilisé par la récupération
lexicale et le chat RAG, qui ne transmettent au modèle que des extraits autorisés
et bornés. La recherche sémantique n'est pas encore connectée à l'API.
