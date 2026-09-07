# Documentation du LLM Security Lab

Ce dossier décrit le laboratoire par sujet. Le flux réellement actif reste la
référence ; les documents « futur » décrivent une étape non activée.

| Document | Contenu |
| --- | --- |
| [system-architecture.md](system-architecture.md) | Vue d'ensemble des composants et frontières. |
| [local-api-architecture.md](local-api-architecture.md) | Routes, données et responsabilités de l'API Python. |
| [api-request-flow.md](api-request-flow.md) | Séquence exacte d'une requête, de l'UI à Ollama. |
| [demo-sso-authentication.md](demo-sso-authentication.md) | Simulation d'identité et de session signée. |
| [rbac-acl-policy.md](rbac-acl-policy.md) | Rôles, ACL et matrice Alice/Bob/Charlie/Oscar. |
| [controlled-document-access.md](controlled-document-access.md) | Lecture de fichiers après ACL et confinement des chemins. |
| [demo-document-catalog.md](demo-document-catalog.md) | Les 15 documents fictifs et leurs classifications. |
| [lexical-rag-retrieval.md](lexical-rag-retrieval.md) | Recherche lexicale active, filtrée par ACL. |
| [rag-generation-flow.md](rag-generation-flow.md) | Chat RAG actif et contexte construit côté serveur. |
| [semantic-rag-design.md](semantic-rag-design.md) | Couche vectorielle préparée, non encore activée. |
| [authorization-security-boundary.md](authorization-security-boundary.md) | Règles d'autorisation actuelles et futures. |
| [security-requirements.md](security-requirements.md) | Exigences réseau, données, audit et privilèges. |
| [security-test-strategy.md](security-test-strategy.md) | Scénarios et preuves de sécurité automatisées. |
| [release-management.md](release-management.md) | CI, livraisons locales et retour arrière. |
| [installation-and-local-services.md](installation-and-local-services.md) | Journal d'installation et services locaux. |
| [mcp-authorization-contract.md](mcp-authorization-contract.md) | Contrat minimal avant tout MCP futur. |

`STATUS.md`, à la racine du dépôt, est un état de reprise local ignoré par Git.
Il ne fait pas partie de cette documentation versionnée.
