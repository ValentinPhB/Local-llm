# Authentification SSO de démonstration

## But pédagogique

Ce laboratoire ne se connecte à aucun annuaire d'entreprise. Il reproduit
néanmoins le contrat technique important d'un SSO : l'API ne reçoit pas une
identité libre dans chaque requête ; elle vérifie un jeton signé qui contient
un sujet et des groupes.

```text
Choix explicite d'une identité fictive dans le navigateur
    -> mini fournisseur d'identité local
    -> JWT HS256 signé, valide 15 minutes, dans un cookie HttpOnly
    -> API du laboratoire vérifie signature, issuer, audience et expiration
    -> groupes fiables -> rôles applicatifs -> décision RBAC/ACL
```

La sélection d'Alice, Bob, Charlie ou Oscar est libre. Elle **ne constitue pas une
authentification réelle** et ne doit jamais être présentée comme telle. Elle
permet de simuler une assertion d'identité et de tester les refus avant de
connecter un futur fournisseur OIDC d'entreprise.

## Contrats versionnés

| Fichier | Rôle |
| --- | --- |
| `config/demo-idp/directory.json` | Annuaire fictif : issuer, audience, durée et groupes des quatre identités. |
| `identity/demo_sso.py` | Émet et vérifie les JWT de démonstration. |
| `config/access-control/demo-policy.json` | Traduit les groupes (`HR`, `IT`…) en rôles applicatifs et définit les ACL. |
| `access_control/engine.py` | Prend la décision déterministe, sans LLM. |

La clé HMAC est créée aléatoirement au démarrage de `ui/server.py`. Elle reste
en mémoire et n'est ni écrite sur disque, ni committée. Redémarrer le serveur
invalide donc toutes les sessions de démonstration existantes.

## Contenu vérifié du jeton

Le JWT contient `iss`, `aud`, `sub`, `groups`, `iat` et `exp`. Avant de l'utiliser,
le serveur vérifie :

- l'algorithme exact `HS256` et la signature ;
- l'émetteur `local-demo-idp` et l'audience `local-llm-lab-api` ;
- l'expiration de 15 minutes ;
- que l'identité et ses groupes existent toujours dans l'annuaire fictif.

Un jeton modifié, expiré, émis pour une autre audience ou contenant des groupes
différents est refusé. Le cookie est `HttpOnly`, `SameSite=Strict` et `Path=/` :
le JavaScript de la page ne peut pas lire le jeton.

L'interface est servie en HTTP sur `127.0.0.1`, donc l'attribut de cookie
`Secure` n'est volontairement pas présent. Cette exception n'est acceptable
que parce que le laboratoire est local et lié à la boucle locale. Toute future
exposition réseau exigera HTTPS et des cookies `Secure`.

## Limites et remplacement futur

Le mini fournisseur d'identité vit dans le même processus que l'API : il ne
simule pas une frontière réseau complète entre un IdP et une application. Il
ne gère ni mot de passe, ni MFA, ni révocation, ni annuaire réel.

Lors d'une intégration entreprise, on supprimera l'écran de choix fictif et
l'émission locale. L'API conservera la même étape de vérification mais fera
confiance aux jetons OIDC de l'IdP choisi, avec ses clés publiques, son issuer,
son audience et ses groupes. Le moteur RBAC/ACL restera indépendant.
