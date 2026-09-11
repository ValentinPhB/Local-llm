# Architecture détaillée de l’API Rust

## Assemblage et responsabilités

| Fichier / module | Responsabilité |
| --- | --- |
| [main.rs](../crates/chatpurp-api/src/main.rs) | Valide les arguments, assemble puis écoute sur 127.0.0.1:3211. |
| [bootstrap.rs](../crates/chatpurp-api/src/bootstrap.rs) | Charge les deux JSON bornés, vérifie leur cohérence avant écoute et construit les adaptateurs. |
| [http.rs](../crates/chatpurp-api/src/http.rs) | Admission HTTP, cookies, routes, corps bornés, réponses publiques ; pas de décision ACL propre. |
| [policy.rs](../crates/chatpurp-core/src/policy.rs) | Annuaire, groupes → rôles → ACL, validation et refus par défaut. |
| [application.rs](../crates/chatpurp-core/src/application.rs) | Session → autorisation → audit → lecture/recherche ou préparation du chat. |
| [sessions.rs](../crates/chatpurp-api/src/sessions.rs) | JWT HS256 avec aws-lc-rs et horloge injectable ; cookies non ambigus. |
| [storage.rs](../crates/chatpurp-api/src/storage.rs) | Lecture par descripteurs rustix ; audit privé avec rotation. |
| [outgoing.rs](../crates/chatpurp-api/src/outgoing.rs) | HTTP sortant loopback, délais, tailles, absence de proxy/redirection ; Ollama. |
| [semantic.rs](../crates/chatpurp-api/src/semantic.rs) | Embeddings, recherche filtrée et writer Qdrant ; non raccordés aux routes. |
| [indexing.rs](../crates/chatpurp-core/src/indexing.rs) | Découpage et validation du lot administratif avant écriture. |
| [assets.rs](../crates/chatpurp-api/src/assets.rs) | Charge les seuls JS/WASM/snippets autorisés et les assets statiques avant écoute. |
| [contracts](../crates/chatpurp-contracts/src/lib.rs) | Types JSON publics sans secret ni chemin interne. |

Les ports Sessions, Reader et Audit sont des interfaces Rust du core, pas des
ports TCP. Les tests y injectent des espions pour prouver l’ordre des appels.
Les lectures disque sont exécutées hors du thread asynchrone HTTP.

## Routes

| Route | Entrée | Sortie utile |
| --- | --- | --- |
| GET /healthz | Aucun corps | status et modèle fixe ; ne prouve pas qu’Ollama répond. |
| POST /api/demo-session | identity_id, JSON ≤ 256 octets | Session fictive et cookie ; aucune identité externe. |
| GET /api/session | Cookie | authenticated, identity.id, identity.display_name. |
| POST /api/logout | Aucun corps | authenticated=false, cookie supprimé. |
| GET /api/documents/<id> | Identifiant brut | resource_id, classification, content. |
| GET /api/access-check?resource_id=<id> | Identifiant unique | allowed=true si admis ; sinon erreur de refus. |
| POST /api/retrieve | query, JSON ≤ 1024 octets | Au plus trois résultats, extrait ≤ 500 caractères. |
| POST /api/chat | message, JSON ≤ 16000 octets | content, sans historique. |
| POST /api/rag-chat | Même entrée | content et identifiants de sources autorisées. |

Les champs supplémentaires non ambigus sont ignorés, jamais des droits.
Les doublons JSON sont rejetés à tous les niveaux, même dans un champ ignoré.
Le filtre HTTP précède le cas d’usage. Les en-têtes communs sont JSON UTF-8,
Cache-Control: no-store et X-Content-Type-Options: nosniff.

## Erreurs documentaires, dans cet ordre

| Statut | Condition et message |
| --- | --- |
| 401 | Session de démonstration requise. |
| 400 | Ressource de démonstration invalide. |
| 403 | Accès au document refusé. Inconnu et interdit sont identiques. |
| 503 | Journal de sécurité indisponible. |
| 500 | Document indisponible. |

Une panne d’audit ne remplace pas un refus déjà acquis. C’est aussi le choix
uniformisé pour access-check en Rust. Une réponse d’erreur ne retourne jamais
un document partiel ou un chemin système. Un problème Ollama retourne 502 avec
un message générique, sans détails de la connexion.

## Admission HTTP et assets

Host unique exactement 127.0.0.1:3211 ; Origin unique exactement
http://127.0.0.1:3211 s’il est fourni, obligatoire pour les POST API.
Aucun repli Referer, Forwarded ou localhost, aucun CORS. Méthode inconnue sur
route connue : 405 et Allow ; route inconnue : 404. HEAD API : 405 sans corps.

Ligne/en-têtes ≤ 16 KiB ; réception des en-têtes puis du corps ≤ 5 secondes
chacune au total. Corps vide sur les GET et logout. Content-Encoding refusé.
JSON : application/json, optionnellement charset=utf-8 seulement.
L’URL doit être relative. Les suffixes documentaires ne sont jamais décodés :
la session précède la validation de l’identifiant brut.

Hyper reste l’unique parseur. Content-Length identiques sont normalisés ;
contradictoires sont rejetés. Un cadrage mixte valide utilise chunked.
Toutes les connexions servent une seule requête : aucune requête pipelinée
suivante ne sera exécutée, y compris après rejet ou cadrage mixte.

Les assets sont chargés dans une table de chemins explicites, limitée à
64 fichiers générés et 32 Mio au total. Le serveur ne publie pas le dépôt.
CSP : scripts locaux et compilation WASM, styles locaux, connexions locales ;
workers, frames, objets, formulaires et inclusion dans une frame interdits.

## Relation avec l’API Ollama

Aucune route d’Ollama n’est « surchargée » ou modifiée dans son serveur.
Notre API est une couche intermédiaire : elle valide, autorise et transforme
la requête, puis appelle l’API officielle locale /api/chat ou, séparément,
/api/embed. Ni le navigateur ni le LLM ne choisissent un endpoint sortant.

Voir les [séquences](api-request-flow.md) et la [sécurité](security-requirements.md).
