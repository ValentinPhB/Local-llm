# Jeu documentaire fictif

## Objet

Les quinze fichiers sous `demo-documents/` sont de vrais fichiers Markdown
versionnés dans ce dépôt. Ils ne contiennent aucune donnée d'entreprise, aucun
nom réel, secret, identifiant ou information personnelle. Ils servent à
apprendre le lien entre une ressource physique, ses métadonnées et sa règle
d'accès.

Ils ne sont pas encore indexés, recherchés ni envoyés à Ollama. L'API peut
retourner un fichier après autorisation ACL, mais elle ne réalise aucune
recherche et ne l'ajoute jamais au prompt. Le contrat du lecteur est décrit dans
[`controlled-document-reader.md`](controlled-document-reader.md).

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

## Étape suivante, non implémentée

Le lecteur documentaire contrôlé est désormais en place. La prochaine couche
sera la récupération RAG filtrée : elle devra interroger uniquement les
ressources autorisées, produire des passages bornés et garantir qu'aucun passage
interdit n'est envoyé à Ollama. Le lecteur continuera de refuser un chemin
absent, inconnu, hors de `demo-documents/` ou non autorisé avant toute lecture.
