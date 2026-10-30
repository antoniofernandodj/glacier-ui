#!/usr/bin/env python3
"""Servidor local para testar o `fetch` (examples/gva/rede_fetch): cookie, eco, upload, rota lenta.

    python3 tools/servidor_rede.py [porta]     # padrão 8096
"""
import re, sys, time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

class H(BaseHTTPRequestHandler):
    def _resp(self, texto, status=200, extra=None):
        corpo = texto.encode()
        self.send_response(status)
        self.send_header("Content-Type", "text/plain; charset=utf-8")
        self.send_header("Content-Length", str(len(corpo)))
        for k, v in (extra or []):
            self.send_header(k, v)
        self.end_headers()
        self.wfile.write(corpo)

    def do_GET(self):
        if self.path == "/set":
            self._resp("cookie definido", extra=[("Set-Cookie", "sid=abc123; Path=/"), ("Set-Cookie", "tema=escuro; Path=/; Max-Age=3600")])
        elif self.path == "/apaga":
            self._resp("cookie apagado", extra=[("Set-Cookie", "sid=; Path=/; Max-Age=0")])
        elif self.path == "/eco":
            self._resp("Cookie: " + self.headers.get("Cookie", "(nenhum)"))
        elif self.path == "/lento":
            time.sleep(4)
            try:
                self._resp("demorou 4s")
            except Exception:
                pass
        else:
            self._resp("não achei", 404)

    def do_POST(self):
        n = int(self.headers.get("Content-Length", 0))
        corpo = self.rfile.read(n)
        if self.path != "/upload":
            return self._resp("não achei", 404)
        tipo = self.headers.get("Content-Type", "")
        m = re.search(r"boundary=(.+)", tipo)
        if not tipo.startswith("multipart/form-data") or not m:
            return self._resp("não é multipart: " + tipo, 400)
        sep = b"--" + m.group(1).encode()
        linhas = []
        for parte in corpo.split(sep)[1:-1]:
            cab, _, dados = parte.partition(b"\r\n\r\n")
            dados = dados[:-2]  # o \r\n antes do próximo boundary
            nome = re.search(rb'name="([^"]*)"', cab)
            arq = re.search(rb'filename="([^"]*)"', cab)
            ct = re.search(rb"Content-Type: ([^\r\n]*)", cab)
            if arq:
                linhas.append(f"arquivo {nome.group(1).decode()}={arq.group(1).decode()} ({len(dados)} bytes, {ct.group(1).decode() if ct else '?'})")
            else:
                linhas.append(f"campo {nome.group(1).decode()}={dados.decode()}")
        self._resp(" | ".join(linhas))

    def log_message(self, *a):
        pass

if __name__ == "__main__":
    porta = int(sys.argv[1]) if len(sys.argv) > 1 else 8096
    ThreadingHTTPServer(("127.0.0.1", porta), H).serve_forever()
