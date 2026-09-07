# Jeu documentaire fictif

## Objet

Les quinze fichiers sous `demo-documents/` sont de vrais fichiers Markdown
versionnés dans ce dépôt. Ils ne contiennent aucune donnée d'entreprise, aucun
nom réel, secret, identifiant ou information personnelle. Ils servent à
apprendre le lien entre une ressource physique, ses métadonnées et sa règle
d'accès.

Ils ne sont pas encore indexés, recherchés ni envoyés à Ollama. À ce stade,
l'API ne retourne que la décision `autorisé` ou `refusé` associée à leur
identifiant : elle ne lit pas le fichier, y compris lorsque l'accès est
autorisé.

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

La prochaine couche sera un lecteur documentaire contrôlé : il recevra un
identifiant de ressource déjà autorisé, lira uniquement le chemin déclaré dans
la politique, puis produira éventuellement des passages pour la récupération
RAG. Il devra refuser un chemin absent, inconnu, hors de `demo-documents/` ou
non autorisé avant toute lecture et avant tout appel à Ollama.
