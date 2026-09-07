#!/usr/bin/env python3
"""Interface locale minimale, identité de démonstration et relais Ollama.

Le faux SSO n'est utilisable que dans ce laboratoire : une personne peut y
choisir librement Alice, Bob ou Charlie. Son objectif est d'exercer le flux
technique « jeton signé -> groupes -> rôles -> ACL », pas de prouver l'identité
d'une personne réelle.
"""

from __future__ import annotations

import json
import re
import sys
from http import HTTPStatus
from http.cookies import SimpleCookie
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any, Mapping
from urllib.error import URLError
from urllib.parse import parse_qs, urlsplit
from urllib.request import Request, urlopen

# L'exécution documentée est ``python3 ui/server.py``. Dans ce mode Python ne
# place que ``ui/`` dans son chemin de modules ; ajouter explicitement la racine
# du dépôt rend les composants d'identité et d'accès disponibles.
PROJECT_ROOT = Path(__file__).resolve().parents[1]
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from access_control.engine import decide_access_for_roles, policy_is_valid, roles_from_groups
from identity.demo_sso import (
    DemoDirectory,
    TokenError,
    VerifiedIdentity,
    issue_demo_token,
    load_demo_directory,
    new_signing_key,
    verify_demo_token,
)


HOST = "127.0.0.1"
PORT = 3210
OLLAMA_CHAT_URL = "http://127.0.0.1:11434/api/chat"
MODEL = "qwen3:4b"
MAX_MESSAGE_CHARS = 8_000
MAX_AUTH_BODY_BYTES = 256
SESSION_COOKIE_NAME = "lab_demo_session"
ROOT = PROJECT_ROOT
INDEX = Path(__file__).with_name("index.html")
POLICY_PATH = ROOT / "config" / "access-control" / "demo-policy.json"
DIRECTORY_PATH = ROOT / "config" / "demo-idp" / "directory.json"
# qwen3 peut commencer la trace sans émettre la balise ouvrante <think>.
# Toute réponse qui contient une fermeture </think> est donc tronquée avant elle.
THINKING_PREFIX = re.compile(r"^.*?</think>\s*", re.DOTALL | re.IGNORECASE)


class ApplicationState:
    """État immuable du processus, chargé et validé avant l'écoute HTTP."""

    def __init__(self, policy: Mapping[str, Any], directory: DemoDirectory) -> None:
        self.policy = policy
        self.directory = directory
        # Clé uniquement en mémoire : redémarrer le serveur ferme toutes les sessions.
        self.signing_key = new_signing_key()


class LocalLabServer(ThreadingHTTPServer):
    """Serveur HTTP local portant l'état validé de l'application."""

    state: ApplicationState


class LocalUIHandler(BaseHTTPRequestHandler):
    server_version = "LocalLLMUI/0.2"

    @property
    def state(self) -> ApplicationState:
        return self.server.state  # type: ignore[attr-defined]

    def log_message(self, format: str, *args: object) -> None:
        """Ne pas écrire les chemins, prompts, réponses ou jetons dans les journaux."""

    def send_json(
        self,
        status: HTTPStatus,
        body: Mapping[str, object],
        extra_headers: Mapping[str, str] | None = None,
    ) -> None:
        payload = json.dumps(body, separators=(",", ":")).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(payload)))
        self.send_header("Cache-Control", "no-store")
        self.send_header("X-Content-Type-Options", "nosniff")
        if extra_headers:
            for name, value in extra_headers.items():
                self.send_header(name, value)
        self.end_headers()
        self.wfile.write(payload)

    def _read_json(self, max_length: int) -> Mapping[str, Any] | None:
        try:
            length = int(self.headers.get("Content-Length", "0"))
            if not 0 < length <= max_length:
                raise ValueError
            body = json.loads(self.rfile.read(length))
            if not isinstance(body, Mapping):
                raise ValueError
            return body
        except (TypeError, ValueError, json.JSONDecodeError):
            return None

    def _session(self) -> VerifiedIdentity | None:
        raw_cookie = self.headers.get("Cookie")
        if not raw_cookie:
            return None
        try:
            cookies = SimpleCookie()
            cookies.load(raw_cookie)
            morsel = cookies.get(SESSION_COOKIE_NAME)
            if morsel is None:
                return None
            return verify_demo_token(morsel.value, self.state.directory, self.state.signing_key)
        except (TokenError, ValueError):
            return None

    def _require_session(self) -> VerifiedIdentity | None:
        session = self._session()
        if session is None:
            self.send_json(HTTPStatus.UNAUTHORIZED, {"error": "Session de démonstration requise."})
        return session

    def _identity_body(self, identity: VerifiedIdentity) -> dict[str, object]:
        directory_identity = self.state.directory.identities[identity.identity_id]
        return {
            "authenticated": True,
            "identity": {
                "id": directory_identity.identity_id,
                "display_name": directory_identity.display_name,
            },
        }

    def _session_cookie(self, token: str) -> str:
        return (
            f"{SESSION_COOKIE_NAME}={token}; Max-Age={self.state.directory.token_ttl_seconds}; "
            "Path=/; HttpOnly; SameSite=Strict"
        )

    def do_GET(self) -> None:  # noqa: N802
        request = urlsplit(self.path)
        if request.path == "/healthz":
            self.send_json(HTTPStatus.OK, {"status": "ok", "model": MODEL})
            return
        if request.path == "/api/session":
            session = self._require_session()
            if session is not None:
                self.send_json(HTTPStatus.OK, self._identity_body(session))
            return
        if request.path == "/api/access-check":
            session = self._require_session()
            if session is None:
                return
            values = parse_qs(request.query, keep_blank_values=True)
            resource_ids = values.get("resource_id", [])
            if len(resource_ids) != 1 or not resource_ids[0]:
                self.send_json(HTTPStatus.BAD_REQUEST, {"error": "Ressource de démonstration invalide."})
                return
            roles = roles_from_groups(self.state.policy, session.groups)
            decision = decide_access_for_roles(self.state.policy, roles, resource_ids[0])
            # Le résultat est volontairement minimal : pas de rôle, de groupe ou de
            # motif détaillé transmis au navigateur.
            status = HTTPStatus.OK if decision.allowed else HTTPStatus.FORBIDDEN
            self.send_json(status, {"resource_id": resource_ids[0], "allowed": decision.allowed})
            return
        if request.path != "/":
            self.send_error(HTTPStatus.NOT_FOUND)
            return

        page = INDEX.read_bytes()
        self.send_response(HTTPStatus.OK)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(page)))
        self.send_header("Cache-Control", "no-store")
        self.send_header("X-Content-Type-Options", "nosniff")
        self.end_headers()
        self.wfile.write(page)

    def do_POST(self) -> None:  # noqa: N802
        request = urlsplit(self.path)
        if request.path == "/api/demo-session":
            body = self._read_json(MAX_AUTH_BODY_BYTES)
            identity_id = body.get("identity_id") if body else None
            if not isinstance(identity_id, str) or not identity_id:
                self.send_json(HTTPStatus.BAD_REQUEST, {"error": "Identité de démonstration invalide."})
                return
            try:
                token = issue_demo_token(self.state.directory, identity_id, self.state.signing_key)
                identity = verify_demo_token(token, self.state.directory, self.state.signing_key)
            except TokenError:
                self.send_json(HTTPStatus.UNAUTHORIZED, {"error": "Identité de démonstration refusée."})
                return
            self.send_json(
                HTTPStatus.OK,
                self._identity_body(identity),
                {"Set-Cookie": self._session_cookie(token)},
            )
            return
        if request.path == "/api/logout":
            self.send_json(
                HTTPStatus.OK,
                {"authenticated": False},
                {"Set-Cookie": f"{SESSION_COOKIE_NAME}=; Max-Age=0; Path=/; HttpOnly; SameSite=Strict"},
            )
            return
        if request.path != "/api/chat":
            self.send_error(HTTPStatus.NOT_FOUND)
            return

        # Le navigateur ne fournit jamais d'identité dans la requête de chat.
        # La session signée est vérifiée avant tout appel à Ollama.
        if self._require_session() is None:
            return
        body = self._read_json(MAX_MESSAGE_CHARS * 2)
        raw_message = body.get("message") if body else None
        if not isinstance(raw_message, str):
            self.send_json(HTTPStatus.BAD_REQUEST, {"error": "Message invalide."})
            return
        message = raw_message.strip()
        if not message or len(message) > MAX_MESSAGE_CHARS:
            self.send_json(HTTPStatus.BAD_REQUEST, {"error": "Message invalide."})
            return

        request_body = json.dumps(
            {
                "model": MODEL,
                "messages": [{"role": "user", "content": message}],
                "stream": False,
                "think": False,
            }
        ).encode("utf-8")
        ollama_request = Request(
            OLLAMA_CHAT_URL,
            data=request_body,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        try:
            with urlopen(ollama_request, timeout=120) as response:  # noqa: S310 -- URL constante locale
                ollama_response = json.loads(response.read())
            content = ollama_response["message"]["content"]
            # qwen3:4b peut ignorer think:false. Ne pas transmettre sa trace.
            content = THINKING_PREFIX.sub("", content).strip()
            self.send_json(HTTPStatus.OK, {"content": content})
        except (URLError, TimeoutError, KeyError, TypeError, json.JSONDecodeError):
            self.send_json(
                HTTPStatus.BAD_GATEWAY,
                {"error": "Ollama local est indisponible ou a renvoyé une réponse invalide."},
            )


def create_server(host: str = HOST, port: int = PORT) -> LocalLabServer:
    """Charge les contrats avant écoute : une configuration invalide échoue tôt."""

    try:
        policy = json.loads(POLICY_PATH.read_text(encoding="utf-8"))
        if not policy_is_valid(policy):
            raise ValueError("politique RBAC/ACL invalide")
        directory = load_demo_directory(DIRECTORY_PATH)
    except (OSError, ValueError, json.JSONDecodeError, TokenError) as error:
        raise RuntimeError("Configuration locale d'identité ou d'accès invalide.") from error
    server = LocalLabServer((host, port), LocalUIHandler)
    server.state = ApplicationState(policy, directory)
    return server


if __name__ == "__main__":
    local_server = create_server()
    print(f"Interface locale : http://{HOST}:{PORT}")
    print("Mode identité : simulation locale, sessions valables 15 minutes.")
    local_server.serve_forever()
