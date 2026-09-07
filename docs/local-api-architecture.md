# Architecture des API locales

## Réponse courte

Le projet ne surcharge pas l'API d'Ollama et ne modifie pas son application.
Il ajoute une **seconde API locale**, écrite en Python, qui sert d'intermédiaire
entre le navigateur et Ollama.

```text
Navigateur  ->  API du laboratoire (Python, port 3210)
                    -> simulation SSO locale : jeton signé + cookie HttpOnly
                    -> groupes -> rôles -> ACL
                    -> API native d'Ollama (port 11434)
                    -> modèle qwen3:4b
```

Les deux services écoutent uniquement sur `127.0.0.1` : ils ne sont pas
accessibles depuis le réseau local ou Internet.

## Les deux API

| API | Processus | Adresse | Rôle |
| --- | --- | --- | --- |
| Ollama | application Ollama | `http://127.0.0.1:11434` | Gère les modèles et exécute l'inférence. |
| Laboratoire | `python3 ui/server.py` | `http://127.0.0.1:3210` | Sert l'interface et applique les contrôles propres au projet. |

L'API Ollama existe dès que l'application Ollama est démarrée. Le téléchargement
de `qwen3:4b` ajoute un modèle utilisable, mais ne crée pas l'API.

L'API du laboratoire existe seulement pendant l'exécution de `ui/server.py`.
Elle n'est pas encore installée comme service permanent.

## Routes exposées par le laboratoire

| Méthode et route | Rôle | Exemple de réponse |
| --- | --- | --- |
| `GET /` | Retourne la page `ui/index.html`. | Interface de conversation. |
| `GET /healthz` | Vérifie que le serveur Python répond. | `{"status":"ok","model":"qwen3:4b"}` |
| `POST /api/demo-session` | Émet une session fictive signée après choix explicite d'Alice, Bob, Charlie ou Oscar. | Cookie `HttpOnly` et identité affichable. |
| `GET /api/session` | Vérifie et retourne l'identité fictive de la session. | `{"authenticated":true,"identity":{…}}` |
| `GET /api/access-check?resource_id=…` | Évalue l'ACL d'un document fictif avec les groupes du jeton, sans lire le fichier. | `{"resource_id":"rh-onboarding","allowed":true}` |
| `POST /api/logout` | Invalide le cookie côté navigateur. | `{"authenticated":false}` |
| `POST /api/chat` | Vérifie d'abord la session, puis envoie le message à Ollama. | `{"content":"…"}` |

La route `POST /api/chat` attend uniquement un JSON de cette forme :

```json
{"message": "Bonjour"}
```

Un message absent, vide, non textuel ou trop long est refusé avec le statut
HTTP `400`. Le navigateur ne peut ni choisir un autre modèle, ni fournir une
URL d'Ollama, ni transmettre un outil.

## Trajet d'un message

```text
1. Le navigateur charge index.html depuis 127.0.0.1:3210.
2. En mode démonstration, il choisit une identité fictive une seule fois.
3. server.py émet un jeton signé temporaire dans un cookie `HttpOnly`.
4. Le JavaScript du bouton « Envoyer » appelle POST /api/chat sans identité libre.
5. server.py vérifie signature, expiration, issuer et audience du jeton.
6. server.py vérifie le format et la taille du message, puis appelle Ollama.
7. Ollama transmet la demande à qwen3:4b et retourne du JSON.
8. server.py retire une éventuelle trace de raisonnement Qwen et renvoie `content`.
```

Le navigateur communique donc avec l'API Python. Python communique ensuite avec
Ollama. Le navigateur ne contacte pas directement le port 11434.

## Ce que Python adapte

Avant l'appel à Ollama, `ui/server.py` impose :

- le modèle `qwen3:4b` ;
- l'adresse locale fixe `http://127.0.0.1:11434/api/chat` ;
- une limite de 8 000 caractères pour le message ;
- `stream: false` : Ollama répond en une seule réponse JSON ;
- `think: false` : demande à Ollama de ne pas produire de raisonnement visible.

Après l'appel, le script retire tout contenu placé avant `</think>`. Cette
protection complémentaire est nécessaire car Qwen peut ignorer `think: false`.
Le script ne journalise ni prompt ni réponse et retourne `502` si Ollama est
indisponible ou renvoie une réponse invalide.

Ces adaptations ont lieu dans **notre** API, après réception de la réponse
d'Ollama. Elles ne changent ni les routes d'Ollama, ni ses modèles, ni ses
fichiers de configuration.

## Ce que Python ne fait pas encore

- pas d'authentification réelle ni de connexion à un annuaire d'entreprise ;
- pas de lecture, d'import ou d'indexation des quinze documents fictifs ;
- pas de base documentaire, import de fichier ou RAG ;
- pas de MCP, d'outil ou de credential ;
- pas de persistance des conversations.

La prochaine évolution ajoutera un lecteur contrôlé des documents fictifs : le
serveur filtrera les ressources avant toute lecture, recherche et tout envoi de
passage à Ollama. Une décision de refus empêchera donc l'envoi du contexte au
modèle.

## Démarrage et vérification

```text
python3 ui/server.py
curl http://127.0.0.1:3210/healthz
curl http://127.0.0.1:11434/api/version
```

Le premier `curl` vérifie l'API Python ; le second vérifie l'API native
d'Ollama. Ce sont deux contrôles distincts.
