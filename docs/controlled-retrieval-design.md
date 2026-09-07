# Récupération contrôlée

## État actuel

La partie **R** de RAG est maintenant implémentée : elle retrouve des extraits
pertinents parmi les documents auxquels l'identité a déjà droit. Aucun extrait
n'est envoyé à Ollama ; la génération augmentée reste un jalon distinct.

Le RAG complet viendra ensuite :

```text
Récupération contrôlée (prochain jalon)
    -> extraits autorisés retournés au navigateur pour vérification

Génération augmentée (jalon suivant)
    -> seuls ces extraits autorisés ajoutés par le serveur au prompt Ollama
```

## Choix initial : recherche lexicale locale

La première implémentation utilisera des mots-clés, pas des embeddings ni une
base vectorielle. Elle découpera les mots de la question et des documents,
comptera leurs correspondances, puis retournera les quelques extraits les mieux
classés.

Ce n'est pas encore une recherche sémantique complète, mais c'est un vrai
contrat de récupération et le meilleur point de départ pédagogique : aucune
dépendance, aucun téléchargement de modèle d'embeddings, aucun index persistant
et un comportement déterministe pour les tests de sécurité.

Une base vectorielle pourra remplacer le classement lexical plus tard, sans
changer l'ordre des contrôles d'accès.

## Flux non négociable

```text
Question de l'utilisateur
    -> session signée vérifiée
    -> groupes -> rôles
    -> sélection des resource_id autorisés par ACL
    -> lecture contrôlée de ces seules ressources
    -> tokenisation et classement local
    -> maximum 3 extraits bornés
    -> réponse de récupération au navigateur
```

Le navigateur ne fournit ni identité, ni rôle, ni chemin, ni liste de documents
à rechercher. Une ressource RH interdite à Oscar n'est donc pas lue, tokenisée,
indexée ou classée pour Oscar.

## Contrat HTTP actif

```text
POST /api/retrieve
{"query":"Comment se passe l’intégration ?"}
```

Entrée : une chaîne non vide, limitée à 500 caractères. Sortie : au plus trois
objets `resource_id`, `classification` et `excerpt`, sans chemin local.

Une session absente renvoie `401`. Une requête invalide renvoie `400`. Une
recherche sans résultat autorisé renvoie `200` avec une liste vide : elle ne
signifie pas qu'un document interdit existe.

## Limites actuelles

- Pas d'index sur disque, de base vectorielle, d'embeddings ou de cloud.
- Pas de filtre fourni par le navigateur ; l'ACL côté serveur est le filtre.
- Pas de contenu de document dans les journaux.
- Pas de transfert vers Ollama et pas de modification de `POST /api/chat`.
- Pas de MCP.

## Preuves obtenues avant la génération augmentée

1. Oscar obtient un extrait de `public-welcome` pour une requête correspondante.
2. Sa liste ACL ne contient que cette ressource ; les autres documents ne sont
   donc pas ouverts par le récupérateur.
3. Bob ne récupère aucun résultat RH, même avec les mots « intégration checklist ».
4. Les instructions de la question ne participent jamais au calcul ACL.
5. Un résultat est borné à trois extraits de 500 caractères maximum.
6. Le chat est inchangé : la route de récupération ne contacte pas Ollama.
