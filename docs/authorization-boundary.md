# Frontière d'autorisation future

## État actuel

Le laboratoire est mono-utilisateur et local. L'interface minimale ne reçoit
ni identité, ni document, ni outil : elle ne peut donc pas encore appliquer de
droits d'accès. Cette simplicité est intentionnelle.

## Architecture à ajouter avant toute donnée réelle

```text
Utilisateur authentifié
    -> session vérifiée
    -> moteur de politique (RBAC / ACL)
    -> recherche parmi les ressources autorisées seulement
    -> passerelle LLM : prompt + passages autorisés
    -> réponse

Outil MCP demandé
    -> passerelle MCP
    -> politique utilisateur + outil + action + paramètres
    -> exécution avec identifiant dédié à privilèges minimaux, ou refus
```

## Règles non négociables

- Un refus est la décision par défaut lorsqu'une identité, une règle ou une
  métadonnée d'accès manque.
- La recherche documentaire filtre les ressources avant de produire des
  passages ou des embeddings transmis au modèle.
- Le texte d'un document, une réponse du modèle ou une instruction de prompt
  ne peuvent ni modifier les permissions ni demander un contournement.
- Le LLM ne reçoit ni clé d'administration, ni accès direct au filesystem,
  ni accès réseau libre.
- Chaque MCP sera enregistré avec son propriétaire, ses actions autorisées,
  les paramètres interdits, un identifiant dédié et une trace d'audit.
- Les permissions sont appliquées côté serveur ; elles ne reposent jamais sur
  un élément de l'interface ou une consigne dans le prompt.

## Ordre de mise en œuvre

1. Définir des identités fictives, rôles et ressources de démonstration.
2. Mettre en place l'authentification et le moteur RBAC / ACL.
3. Ajouter l'import de documents avec propriétaire et ACL obligatoires.
4. Construire la récupération filtrée et prouver les refus avec les scénarios
   `SEC-01` à `SEC-05`.
5. Ajouter un seul MCP de démonstration, sans privilège système, puis prouver
   que les actions interdites sont refusées.

Aucune de ces étapes ne sera activée sans test de refus correspondant.
