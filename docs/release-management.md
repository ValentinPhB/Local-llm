# Release management du LLM Security Lab

## But

Le laboratoire contient plusieurs composants locaux qui communiquent : Ollama,
le modèle, l'API Python et, plus tard, la politique RBAC/ACL, la récupération
documentaire et les MCP. Cette procédure permet de savoir **quoi** a changé,
**comment** cela a été vérifié et **comment revenir en arrière**.

À ce stade, il ne s'agit pas encore d'une plateforme de microservices complète :
tous les composants résident sur le même Mac et aucun n'est exposé sur le
réseau. Les frontières API existent néanmoins déjà et doivent être gérées avec
la même rigueur.

## Composants à versionner

| Composant | Référence à conserver | Exemple actuel |
| --- | --- | --- |
| Code du laboratoire | commit Git, puis tag de release | `main` contient `ui/server.py` et `ui/index.html` |
| API Python | commit Git et version du protocole | `LocalLLMUI/0.1` |
| Ollama | version de l'application | `0.33.3` |
| Modèle | nom, tag, identifiant de contenu et quantification | `qwen3:4b`, ID `359d7dd4bcda`, Q4_K_M |
| Politique RBAC/ACL | schéma et commit Git | schéma `1.0` dans `demo-policy.json` |
| MCP futur | version, origine, permissions et identifiant dédié | non installé |

Un tag de modèle seul, par exemple `qwen3:4b`, est pratique mais ne suffit pas
à une reproductibilité parfaite : un tag peut évoluer. L'identifiant de contenu
observé et la date d'installation doivent donc aussi être consignés.

## Cycle d'une release

```text
Préparer
    -> Examiner les versions, dépendances et risques
    -> Modifier un composant cohérent
    -> Tester les comportements normaux et les refus de sécurité
    -> Documenter la version et le retour arrière
    -> Commit Git
    -> Publier sur main
```

Une release n'est validée que lorsque toutes les étapes sont accomplies.

## Règles avant une mise à jour

1. Définir précisément le composant, l'ancienne version et la version cible.
2. Lire les changements de sécurité et les CVE pertinentes, lorsque le
   composant provient d'un tiers.
3. Vérifier que le changement ne crée pas d'écoute sur le LAN, n'ajoute pas de
   secret et n'accorde pas de nouvelle permission implicite.
4. Préparer la méthode de retour arrière avant d'appliquer le changement.
5. Prévenir si une décision exige un arbitrage : nouveau modèle, exposition
   réseau, stockage persistant, accès documentaire ou MCP.

## Vérifications minimales après une release

| Composant modifié | Vérification minimale |
| --- | --- |
| API Python | syntaxe, `/healthz`, écoute uniquement sur `127.0.0.1`, réponse et refus `400` |
| Ollama | version, `/api/version`, écoute sur `127.0.0.1:11434`, liste de modèles |
| Modèle | `ollama show`, test non sensible, observation mémoire et espace disque |
| Politique RBAC/ACL | matrice autorisation/refus complète, refus par défaut |
| Document/RAG futur | absence de passage non autorisé avant l'appel au LLM |
| MCP futur | autorisations minimales, action autorisée, action interdite, audit |

Un contrôle qui échoue bloque la release, sauf décision explicite documentant
le risque accepté. « Cela fonctionne » ne suffit pas : les refus attendus
doivent aussi fonctionner.

## Retour arrière

| Changement | Retour arrière prévu |
| --- | --- |
| Code Python ou politique | revenir au commit Git connu et redémarrer l'interface |
| Mise à jour Ollama | réinstaller la version précédente vérifiée, puis contrôler l'API locale |
| Ajout ou mise à jour de modèle | arrêter le modèle ; conserver ou retirer l'artefact seulement après décision explicite |
| MCP futur | désactiver ou déconnecter le MCP et révoquer son identifiant dédié |

La suppression d'un modèle, d'un document ou de données persistantes est une
action distincte : elle n'est jamais implicite dans un retour arrière.

## Traçabilité de chaque release

Pour chaque changement significatif, enregistrer dans le commit et, si utile,
dans `STATUS.md` :

- date, composant et versions avant/après ;
- raison du changement ;
- risques examinés ;
- tests exécutés et résultats ;
- méthode de retour arrière ;
- décision éventuelle d'acceptation de risque.

Quand le laboratoire aura une première version stable et reproductible, nous
créerons un tag Git explicite, par exemple `lab-v0.1.0`, accompagné de notes de
release. Nous ne créons pas ce tag aujourd'hui : l'authentification et le
moteur RBAC ne sont pas encore implémentés.
