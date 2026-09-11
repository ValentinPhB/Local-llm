# Plan d’implémentation et contrat de transport

## Périmètre et état

SPEC-001 couvre sessions fictives, lecture contrôlée et audit ; SPEC-002 ajoute
chat, récupération lexicale et capacités sémantiques préparées. MCP exclus.
Le workspace Rust et le parcours navigateur sont implémentés et testés.
La bascule du service actif et la publication restent des opérations distinctes.
Les essais abandonnés ne sont pas conservés comme archives.

## Structure retenue

Quatre packages : chatpurp-core (politique et cas d’usage), chatpurp-contracts
(types publics), chatpurp-api (adapters natifs et exécutables), chatpurp-web
(Dioxus/WASM). web dépend seulement de contracts ; API dépend de core/contracts.
Core n’a ni réseau, ni filesystem natif : Sessions, Reader et Audit sont injectés.

L’assemblage valide deux fichiers JSON de schéma 1.0, de 64 KiB maximum chacun,
sans doublons. La politique est deny par défaut et cohérente avec l’annuaire :
identités, groupes, rôles et ressources uniques. Aucun état autoritaire client.
Les modules et versions sont décrits dans [l’API](../../docs/local-api-architecture.md)
et [release management](../../docs/release-management.md).

## Contrats des quatre routes de SPEC-001

POST /api/demo-session : objet JSON de 256 octets maximum avec identity_id.
Champ absent, vide, mauvais type ou enveloppe invalide : 400,
« Identité de démonstration invalide. ». Toute chaîne non vide inconnue,
sans trim ni changement de casse : 401, « Identité de démonstration refusée. ».
Extras non ambigus ignorés. Succès 200 :
authenticated=true et identity contenant id/display_name de l’annuaire.
Cookie chatpurp_demo_session, Max-Age=900, Path=/, HttpOnly, SameSite=Strict,
sans Domain ni Secure sur ce HTTP loopback. Aucun cookie émis sur refus.

GET /api/session : aucun corps ; même JSON public si valide, sinon 401,
« Session de démonstration requise. ». Pas de réémission ni prolongation.
POST /api/logout : aucun corps ; 200 et authenticated=false, cookie du même
nom/chemin supprimé avec Max-Age=0. Fonctionne sans session. Une copie d’un
jeton n’est pas révoquée par la déconnexion.

GET /api/documents/<id> : aucun corps. Retourne resource_id, classification
et content, y compris front matter, comme texte UTF-8. Priorités après transport :

| Rang | Condition | Statut / message |
| --- | --- | --- |
| 1 | Session absente/invalide | 401 — Session de démonstration requise. |
| 2 | Identifiant invalide | 400 — Ressource de démonstration invalide. |
| 3 | Inconnu/interdit | 403 — Accès au document refusé. |
| 4 | Autorisé, audit échoué | 503 — Journal de sécurité indisponible. |
| 5 | Fichier invalide/indisponible | 500 — Document indisponible. |

Un refus conserve son statut même si l’audit échoue ; aucune égalité de temps
de réponse entre inconnu et interdit n’est promise. Les paramètres de requête
et en-têtes d’identité/droits sont ignorés.

## HTTP-01 accepté

Ordre : parseur/bornes → Host → Origin → route/méthode → enveloppe du corps →
cas d’usage. Un rejet de transport n’appelle ni session, ni audit métier,
ni lecteur, ni émetteur de cookie.

- Host unique exactement 127.0.0.1:<port configuré>, après espaces HTTP périphériques.
  Pas de localhost, autre port, absence, doublon ou autorité Forwarded.
- Origin API unique exactement http://127.0.0.1:<port configuré>.
  Obligatoire sur POST, facultatif sur GET ; null/liste/étranger refusés.
  Pas de repli Referer et aucun CORS.
- Cible relative uniquement. Le préfixe brut /api/documents/ entre dans le cas
  d’usage même avec suffixe vide ou séparateurs : session avant identifiant.
  Aucun décodage de %, normalisation de points ou extraction permissive.
- Identifiant documentaire : [a-z0-9-], longueur 1..80.
- Route inconnue : 404 ; connue, mauvaise méthode : 405 + Allow.
  HEAD API : 405 sans corps et sans appel implicite à GET.
- Doublon de cookie de session même identique ou syntaxe Cookie invalide :
  session invalide. Autres cookies bien formés ignorés. Création/logout
  n’exigent pas un ancien cookie valide.
- Création JSON UTF-8, objet racine, clés uniques à tous les niveaux.
  Content-Type application/json avec éventuellement charset=utf-8 seulement,
  comparaisons insensibles à la casse ; Content-Encoding toujours absent.
- Ligne/en-têtes cumulés : 16 KiB. Délais totaux : en-têtes 5 s, puis corps 5 s.
  Corps reçu/décodé réellement limité, y compris chunked.
- Content-Length identiques et valides sans Transfer-Encoding : normalisés.
  Contradictoires : refus et fermeture avant application.
- Content-Length avec chunked valide : Hyper retire Content-Length et utilise
  chunked. Fermeture après une réponse, aucune deuxième requête exécutée.
  Toutes les connexions sont limitées à une requête dans l’implémentation.
- Pas de second parseur HTTP maison. Si Hyper rejette avant notre filtre,
  fermeture ou erreur native sans donnée sensible admise, sans promesse JSON.

Erreurs du filtre : 400 « Requête HTTP invalide. » (sauf création),
403 « Requête HTTP refusée. », 404 « Route introuvable. »,
405 « Méthode non autorisée. », 415 « Format de requête non pris en charge. ».
Réponses API : JSON UTF-8, no-store, nosniff. Pas de logs des chemins bruts,
en-têtes ou corps. Après rejet, la connexion ne réutilise jamais un corps restant.

Ces règles durcissent l’ancienne API Python. La normalisation du cadrage
par Hyper est un arbitrage accepté ; les tests vérifient les deux ordres
CL/TE, 256/257 octets, refus ACL et compteur prouvant zéro seconde exécution.

## Sessions, stockage et audit

Clé aléatoire 32 octets, HS256, issuer/audience et groupes exacts.
iat/exp entiers non négatifs, exp−iat=900, expiration dès now≥exp.
Horloge injectée côté serveur, aucune confiance dans une date client.
Une erreur d’horloge ou un dépassement arithmétique n’émet pas de jeton.
iat n’est pas un not-before ; un recul d’horloge peut rétablir une validité.

Lecture par descripteur racine et openat NOFOLLOW à chaque composant ;
répertoires intermédiaires obligatoires. Fichier final NONBLOCK, régulier par
fstat, ≤ 32768 octets puis lecture bornée ; UTF-8 et front matter cohérents.
Pas de garantie contre une modification concurrente du contenu régulier par
le même compte système.

Audit Rust distinct, .local/rust-api/audit : 0700/0600, 1 MiB + une sauvegarde.
Six champs minimaux, aucun contenu/secret ; audit d’autorisation avant accès.
Les détails et limites sont centralisés dans [sécurité](../../docs/security-requirements.md).

## Livraison et retour arrière

Rust remplace Python après bascule approuvée. Il utilise
3211, un cookie de nom distinct et un audit distinct ; changer seulement
le port n’isole pas les cookies. Aucun service Ollama/Qdrant n’est remplacé.
Le frontend est servi par la même origine que l’API Rust.

La [suite de preuves](../../docs/security-test-strategy.md) contrôle le métier,
les adaptateurs, le transport brut, le navigateur réel et Qdrant éphémère.
La livraison retire les sources Python remplacées et les guides redondants,
puis commit/push et vérification du workflow sur ce commit. Retour arrière :
version publiée précédente dans un dossier distinct, jamais reset du worktree.
Le rapport OneNote est préparé localement ; transfert après accès/destination.
