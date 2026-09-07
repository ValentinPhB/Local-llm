# Frontière d'autorisation future

## État actuel

Le laboratoire est local et propose trois identités fictives. Il émet un jeton
signé de courte durée, le vérifie côté serveur et transforme ses groupes en
rôles avant d'évaluer les ACL de quinze documents fictifs. Le navigateur ne
fournit pas d'identité à la route de chat.

Ce n'est pas une authentification réelle : le choix d'identité est libre et
sert uniquement à simuler le contrat d'un SSO. Les documents fictifs existent
dans le dépôt, mais aucun n'est encore lu par l'API ; il n'existe ni outil ni
MCP.

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
- Le rôle futur `mcp_read_only` d'Oscar ne pourra autoriser que l'action exacte
  `read` d'un MCP explicitement enregistré. Un MCP, une action ou un paramètre
  inconnu est refusé. Voir [`mcp-authorization.md`](mcp-authorization.md).

## Ordre de mise en œuvre

1. Définir des identités fictives, rôles et ressources de démonstration. **Fait.**
2. Mettre en place une simulation de jeton signé et le moteur RBAC / ACL. **Fait ; pas une authentification réelle.**
3. Ajouter des documents fictifs avec métadonnées et ACL obligatoires. **Fait :
   15 fichiers versionnés, mais pas encore lus par l'API.**
4. Construire le lecteur et la récupération filtrée, puis prouver les refus avec les scénarios
   `SEC-01` à `SEC-05`.
5. Ajouter un seul MCP de démonstration, sans privilège système, puis prouver
   que les actions interdites sont refusées.

Aucune de ces étapes ne sera activée sans test de refus correspondant.
