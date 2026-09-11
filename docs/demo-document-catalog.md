# Catalogue des quinze documents fictifs

Les fichiers de demo-documents sont de vrais fichiers versionnés, exclusivement
fictifs : 9 PUBLIC (60 %), 3 RH (20 %), 3 IT (20 %).
L'identifiant, le chemin et la classification doivent correspondre à la
politique. Le lecteur Rust vérifie ces métadonnées avant de retourner le contenu.

| Identifiant | Classification | Fichier |
| --- | --- | --- |
| public-welcome | PUBLIC | [welcome.md](../demo-documents/public/welcome.md) |
| public-model-guidelines | PUBLIC | [model-guidelines.md](../demo-documents/public/model-guidelines.md) |
| public-incident-reporting | PUBLIC | [incident-reporting.md](../demo-documents/public/incident-reporting.md) |
| public-meeting-rooms | PUBLIC | [meeting-rooms.md](../demo-documents/public/meeting-rooms.md) |
| public-security-basics | PUBLIC | [security-basics.md](../demo-documents/public/security-basics.md) |
| public-remote-work | PUBLIC | [remote-work.md](../demo-documents/public/remote-work.md) |
| public-product-overview | PUBLIC | [product-overview.md](../demo-documents/public/product-overview.md) |
| public-glossary | PUBLIC | [glossary.md](../demo-documents/public/glossary.md) |
| public-training-calendar | PUBLIC | [training-calendar.md](../demo-documents/public/training-calendar.md) |
| rh-onboarding | RH | [onboarding.md](../demo-documents/rh/onboarding.md) |
| rh-leave-policy | RH | [leave-policy.md](../demo-documents/rh/leave-policy.md) |
| rh-benefits | RH | [benefits.md](../demo-documents/rh/benefits.md) |
| it-workstation | IT | [workstation.md](../demo-documents/it/workstation.md) |
| it-password-reset | IT | [password-reset.md](../demo-documents/it/password-reset.md) |
| it-service-catalog | IT | [service-catalog.md](../demo-documents/it/service-catalog.md) |

PUBLIC requiert normalement lab_reader. public-welcome admet aussi
public_welcome_reader, le rôle documentaire d'Oscar. Oscar n'accède à aucun
autre fichier PUBLIC ; son rôle MCP futur n'ajoute aucun droit documentaire.
La [matrice](security-requirements.md) est vérifiée sur les 60 couples
identité/ressource.

Les fichiers peuvent être lus directement ou recherchés lexicalement après
autorisation et audit. Aucun fichier du lab n'a été indexé dans Qdrant pendant
la migration. La préparation sémantique est décrite dans
[recherche et indexation](semantic-rag-design.md).
