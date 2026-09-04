#!/usr/bin/env python3
"""Interface locale minimale pour Ollama.

Le serveur ne sert que ses fichiers statiques et proxyfie un message vers
Ollama sur la boucle locale. Il ne conserve ni prompt ni réponse.
"""

from __future__ import annotations

import json
import re
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.error import URLError
from urllib.request import Request, urlopen


HOST = "127.0.0.1"
PORT = 3210
OLLAMA_CHAT_URL = "http://127.0.0.1:11434/api/chat"
MODEL = "qwen3:4b"
MAX_MESSAGE_CHARS = 8_000
INDEX = Path(__file__).with_name("index.html")
# qwen3 peut commencer la trace sans émettre la balise ouvrante <think>.
# Toute réponse qui contient une fermeture </think> est donc tronquée avant elle.
THINKING_PREFIX = re.compile(r"^.*?</think>\s*", re.DOTALL | re.IGNORECASE)


class LocalUIHandler(BaseHTTPRequestHandler):
    server_version = "LocalLLMUI/0.1"

    def log_message(self, format: str, *args: object) -> None:
        """Ne pas écrire les chemins, prompts ou réponses dans les journaux."""

    def send_json(self, status: HTTPStatus, body: dict[str, object]) -> None:
        payload = json.dumps(body).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(payload)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(payload)

    def do_GET(self) -> None:  # noqa: N802
        if self.path == "/healthz":
            self.send_json(HTTPStatus.OK, {"status": "ok", "model": MODEL})
            return
        if self.path != "/":
            self.send_error(HTTPStatus.NOT_FOUND)
            return

        page = INDEX.read_bytes()
        self.send_response(HTTPStatus.OK)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(page)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(page)

    def do_POST(self) -> None:  # noqa: N802
        if self.path != "/api/chat":
            self.send_error(HTTPStatus.NOT_FOUND)
            return

        try:
            length = int(self.headers.get("Content-Length", "0"))
            if not 0 < length <= MAX_MESSAGE_CHARS * 2:
                raise ValueError
            body = json.loads(self.rfile.read(length))
            raw_message = body["message"]
            if not isinstance(raw_message, str):
                raise ValueError
            message = raw_message.strip()
            if not message or len(message) > MAX_MESSAGE_CHARS:
                raise ValueError
        except (KeyError, TypeError, ValueError, json.JSONDecodeError):
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
        request = Request(
            OLLAMA_CHAT_URL,
            data=request_body,
            headers={"Content-Type": "application/json"},
            method="POST",
        )

        try:
            with urlopen(request, timeout=120) as response:  # noqa: S310 -- URL constante locale
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


if __name__ == "__main__":
    print(f"Interface locale : http://{HOST}:{PORT}")
    ThreadingHTTPServer((HOST, PORT), LocalUIHandler).serve_forever()
