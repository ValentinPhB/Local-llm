# ADR-0002 — Contrats au format SpecDD

Statut : pilote accepté et utilisé pour SPEC-001 et SPEC-002.

Les comportements sont décrits en .sdd, avec un nom identique au répertoire
pour une découverte indépendante du nom du clone. Must porte les exigences,
Done when leurs preuves attendues et Tasks les tâches. Les plans, guides,
décisions et AGENTS.md restent en Markdown.

La CLI officielle verrouillée valide syntaxe et découverte. Un test vérifie
l’appariement des identifiants : 14 pour SPEC-001, 7 pour SPEC-002.
Voir [outillage SpecDD](../../tools/specdd/README.md).

Aucun bootstrap global ni plugin SpecDD n’est installé. Les sections d’un
contrat ne créent pas des permissions système. AGENTS.md reste l’entrée des
règles de collaboration. Le lint n’est pas une preuve que le code implémente
les exigences : les tests Rust/HTTP/navigateur sont indispensables.
