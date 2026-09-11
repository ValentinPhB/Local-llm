# Tests automatisés et preuves

Pas de recette manuelle d’acceptation. Les décisions métier humaines restent
distinctes des vérifications techniques. Les commandes reproductibles sont dans
[release management](release-management.md).

## Exigences et tests

| Contrat | Preuve principale |
| --- | --- |
| SES-01/02/03 | sessions.rs et http/tests.rs : émission, groupe/signature/algorithme, dates, horloge, cookies, expiration et redémarrage. |
| ACL-01/02/03 | policy.rs : matrice 4 × 15 ; application.rs : aucun lecteur après refus ; matrice complète au travers d’Hyper. |
| DOC-01/02/03 | storage.rs : 15 fichiers, taille, Unicode, métadonnées, traversée, symlinks, répertoire et FIFO refusés. |
| AUD-01/02 | Espions d’ordre, échec avant contenu/inférence ; audit réel privé, borné, rotation et refus des liens. |
| HTTP-01 | Octets bruts via le même Hyper que le serveur : Host/Origin, doublons, JSON, méthodes, limites, deadlines et fermeture sans deuxième appel. |
| UI-01/02 | Puppeteer lance le WASM réel : les quatre identités, lectures/refus, recherche, chat/RAG, logout et HTML hostile inerte. |
| ISO-01 | Bootstrap loopback ; tests utilisant doubles de modèle et ports de fixtures ; aucun service du lab dans la suite déterministe. |
| CHAT-01/02 | Espion du fournisseur : zéro appel sans session/audit ; limites et retrait du préfixe think, aucune autorité client. |
| RET-01/RAG-01 | Lectures autorisées seulement, limites, ordre stable, sources serveur, aucun contexte client. |
| NET-01 | Client HTTP réel contre flux de test : taille, compression, JSON invalide, redirection et annulation du pilote à l’expiration. |
| SEM-01 | Vecteurs/lot/découpage, filtre mandatory et payload revérifié ; test réel du writer/recherche sur Qdrant éphémère. |

Liens : [core](../crates/chatpurp-core/src/application.rs),
[HTTP](../crates/chatpurp-api/src/http/tests.rs),
[stockage](../crates/chatpurp-api/src/storage.rs),
[sémantique](../crates/chatpurp-api/src/semantic.rs),
[navigateur](../tools/browser-tests/scenarios/app.test.mjs).

## Couches complémentaires

1. Métier pur et adaptateurs : tests Rust, horloges privées et fichiers temporaires.
2. Transport : vrais encodeur/parseur HTTP avec flux bidirectionnels de test ;
   pas une simple comparaison de fonctions ou d’extracteurs.
3. Navigateur : véritable serveur TCP éphémère et application WASM compilée,
   modèle fictif, profil neuf, cookie HttpOnly et contenu texte.
4. Qdrant : moteur réel dans un conteneur isolé, pas une substitution de réponse.
5. Hygiène : liens, JSON, gitignore, frontières Cargo et absence de code serveur
   dans le graphe de compilation du navigateur.
6. Outillage : pré-vol JS Dioxus, transformation WASM répétable et rejets
   d’artefacts non conformes, politiques navigateur et erreurs d’audit testées.
7. Intégration locale explicite : [live-smoke.mjs](../tools/live-smoke.mjs),
   session/ACL/chat/RAG avec Ollama réel. Hors suite déterministe et hors CI ;
   vérifie le contrat d'une réponse, pas sa qualité rédactionnelle ou factuelle.

La liste des tests et leurs résultats se calcule en exécutant la suite, pas en
maintenant un total figé dans plusieurs guides. Les tests paramétrés couvrent
plusieurs cas chacun ; un nombre de tests ne mesure pas à lui seul la sécurité.

## Limites

Un succès local n’est pas un succès CI distant. Vérifier le workflow du commit
livré dans [GitHub Actions](https://github.com/ValentinPhB/ChatPurp/actions/workflows/tests.yml) :
un ancien résultat vert ne qualifie pas un changement ultérieur.
Les tests ne constituent ni un pentest exhaustif ni une preuve d’inviolabilité
du sandbox OS. Les tests déterministes de chat ne prouvent pas la factualité
du modèle réel. Les contrôles Host/Origin ne résistent pas à un programme local
capable de fabriquer ses propres requêtes.

Les audits Rust et navigateur échouent si les données sont indisponibles ou
incomplètes, ou si des avis sont retournés. SpecDD utilise une politique distincte :
alertes hautes/critiques actives bloquantes, exclusions des paquets dev amont
absents vérifiées et affichées dans son guide d’outillage. Une panne réseau
ne doit pas être convertie en « aucune vulnérabilité ». Les licences des dépendances restent celles des
fournisseurs ; aucun fichier de licence du projet ne doit être inventé.
