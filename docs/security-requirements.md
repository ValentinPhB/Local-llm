# Sécurité, identité et autorisations

## Frontière de confiance

Seule l’API décide des droits. Le navigateur, le prompt et les documents sont
non fiables. Les modèles n’ont ni accès direct au disque, ni clé, ni outil.
Aucun annuaire réel, import personnel, fournisseur cloud ou MCP n’est installé.
L’écoute est loopback seulement ; ce lab ne constitue pas une solution
multiutilisateur sécurisée contre d’autres processus du même compte macOS.

## Identités fictives et ACL

| Identité | Groupes | Rôles | Documents autorisés |
| --- | --- | --- | --- |
| Alice | LAB_READERS, HR | lab_reader, rh_reader | 9 PUBLIC + 3 RH |
| Bob | LAB_READERS, IT | lab_reader, it_reader | 9 PUBLIC + 3 IT |
| Charlie | LAB_READERS | lab_reader | 9 PUBLIC |
| Oscar | OSCAR_PUBLIC_WELCOME_READERS, MCP_READERS | public_welcome_reader, mcp_read_only | public-welcome seulement |

Les références sont [l’annuaire](../config/demo-idp/directory.json) et
[la politique](../config/access-control/demo-policy.json). Refus par défaut.
PUBLIC n’autorise ni l’anonymat ni automatiquement toutes les identités.
RBAC traduit des groupes en rôles ; l’ACL d’une ressource liste les rôles admis.
Déposer un fichier dans un dossier ne crée pas de règle d’accès.

## Sessions

JWT HS256 avec jsonwebtoken 11.0.0 et fournisseur aws-lc-rs 1.18.1, sans connexion
AWS ni prétention FIPS. Clé aléatoire de 32 octets uniquement en mémoire.
Issuer et audience viennent de l’annuaire ; sujet/groupes doivent correspondre
exactement. iat/exp sont des entiers UTC non négatifs ; exp−iat=900 et
now≥exp signifie expiration, sans marge. Une erreur d’horloge refuse l’opération.

iat n’est pas un not-before. L’heure système peut reculer et rendre à nouveau
valide un jeton antérieurement expiré : limite explicite du lab.
Cookie chatpurp_demo_session, HttpOnly, SameSite=Strict, Path=/, sans Domain.
Pas de Secure sur HTTP loopback ; aucune exposition réseau n’est permise
sans contrat HTTPS distinct. Cookies de session multiples ou syntaxe ambiguë :
refus. La déconnexion supprime le cookie, pas une copie du JWT. Le redémarrage
change la clé et invalide les sessions du processus précédent.

## Lecture contrôlée

L’API valide la session avant l’identifiant, puis l’ACL avant le lecteur.
Identifiant : 1 à 80 caractères ASCII minuscules/chiffres/tirets.
Les fichiers viennent uniquement des chemins déclarés dans demo-documents.
Le lecteur conserve un descripteur racine et ouvre chaque composant via openat
avec NOFOLLOW ; les répertoires intermédiaires doivent être des répertoires.
Le fichier final est ouvert sans blocage puis vérifié par fstat : régulier,
taille ≤ 32768 octets ; la lecture elle-même est bornée à 32769 octets.
UTF-8 et métadonnées id/classification sont contrôlés, doublons refusés.

Cela réduit les courses sur les liens symboliques et les changements de chemins.
Cela n’empêche pas un autre processus du même utilisateur de modifier
simultanément le contenu d’un fichier régulier ou le programme lui-même.
Ce n’est ni un sandbox OS ni un contrôle des permissions du compte macOS.

## Audit minimal

Chemin Rust : .local/rust-api/audit/access-decisions.jsonl ; une sauvegarde
access-decisions.1.jsonl. Répertoire 0700, fichiers 0600, 1 MiB maximum chacun.
Fichiers spéciaux et liens refusés ; les fichiers d’audit doivent aussi n’avoir
qu’un lien physique. Écritures sérialisées, lignes JSON terminées et flushées.
Pas de promesse de survie à une panne électrique : il n’y a pas de fsync par ligne.

Champs : timestamp UTC, event=access_decision, route normalisée, outcome,
identity_id vérifié ou nul, resource_id valide ou nul. Pas de token, cookie,
prompt, requête de recherche, source, extrait ou réponse. L’événement décrit
une décision, pas le succès ultérieur de lecture ou d’inférence.

Une autorisation exige l’écriture réussie avant lecture ou appel Ollama.
Un refus tente l’audit mais reste refusé si le journal est indisponible.
Aucune attribution à l’identité revendiquée par un JWT non vérifié.

## Réseau et dépendances

Le [profil HTTP](local-api-architecture.md) est plus strict que l’ancienne API.
Les appels sortants utilisent des sockets loopback explicites, sans proxy ni
redirection ; délais et tailles sont bornés. Qdrant et Ollama ne sont pas
accessibles par les routes du navigateur. Ils restent accessibles à un autre
programme local : le loopback n’est pas une authentification.

Les audits de dépendances vérifient les versions verrouillées ; une absence
d’avis retourné ne prouve pas l’absence de vulnérabilité. Caches, journaux,
binaires, modèles, STATUS.md et secrets restent hors Git.
Un compte ou document réel exige une nouvelle conception de sécurité.

## MCP exclus

Le rôle futur d’Oscar mcp_read_only est conservé dans la politique mais aucun
connecteur n’existe. Un futur MCP devra être explicitement approuvé, chaque
action déclarée et autorisée côté serveur. Oscar ne pourrait recevoir que read ;
écriture, exécution, administration, suppression et actions inconnues seraient
refusées. Ce rôle n’accorde aucun droit documentaire supplémentaire.
