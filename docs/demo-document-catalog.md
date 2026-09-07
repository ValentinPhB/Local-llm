# Catalogue des documents de démonstration

## Objet

Les quinze fichiers sous `demo-documents/` sont de vrais fichiers Markdown
versionnés dans ce dépôt. Ils ne contiennent aucune donnée d'entreprise, aucun
nom réel, secret, identifiant ou information personnelle. Ils servent à
apprendre le lien entre une ressource physique, ses métadonnées et sa règle
d'accès.

Ils ne sont pas indexés dans une base vectorielle. La récupération lexicale
active les recherche seulement après ACL, et `POST /api/rag-chat` peut envoyer
à Ollama des extraits autorisés et bornés. L'API peut aussi retourner un fichier
après autorisation ACL. Le contrat du lecteur est décrit dans
[`controlled-document-access.md`](controlled-document-access.md).

## Répartition imposée

| Classification | Dossier | Nombre | Rôle ACL requis |
| --- | --- | ---: | --- |
| PUBLIC | `demo-documents/public/` | 9 (60 %) | `lab_reader` |
| RH | `demo-documents/rh/` | 3 (20 %) | `rh_reader` |
| IT | `demo-documents/it/` | 3 (20 %) | `it_reader` |
| Total | `demo-documents/` | 15 | — |

Chaque document commence par des métadonnées simples :

```text
---
id: rh-onboarding
classification: RH
owner: demo-rh
---
```

L'identifiant, la classification et le chemin doivent correspondre à une entrée
de `config/access-control/demo-policy.json`. Le test
`tests/test_demo_documents.py` vérifie cette correspondance ainsi que la
répartition 9/3/3. En cas de différence, le test échoue : un fichier ne devient
donc pas silencieusement accessible parce qu'il a été déposé dans un dossier.

## État de récupération

La récupération lexicale et le RAG contrôlé sont en place. L'évolution vers une
base vectorielle ne change pas la règle fondamentale : l'indexeur ne prend que
des ressources de politique et la recherche devra filtrer Qdrant par les
ressources autorisées avant de retourner un passage. Le lecteur continue de
refuser un identifiant absent, inconnu, assimilable à un chemin ou non autorisé
avant toute lecture.
