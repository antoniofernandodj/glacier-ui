#!/usr/bin/env python3
"""Servidor local para testar a rede do Glacier (examples/gva/rede_completa): cookies, eco JSON,
upload multipart, rota lenta, falhas com contagem (retries), status e download.

    python3 tools/servidor_rede.py [porta]     # padrão 8096
"""
import json, re, sys, time
from urllib.parse import urlparse, parse_qs
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

CONTADORES = {}


def eco_json(h, corpo=b""):
    u = urlparse(h.path)
    pegar = ("x-api-key", "x-trace", "authorization", "accept", "content-type", "cookie")
    dados = {
        "method": h.command,
        "path": u.path,
        "query": {k: v[0] for k, v in parse_qs(u.query).items()},
        "headers": {k: h.headers.get(k) for k in pegar if h.headers.get(k) is not None},
        "body": corpo.decode("utf-8", "replace"),
    }
    h._resp(json.dumps(dados), extra=[("Content-Type", "application/json")])


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

    def _caminho(self):
        # `/v1/...` é o mesmo que `/...`: serve para testar `api:extend("/v1")`.
        return re.sub(r"^/v1", "", urlparse(self.path).path)

    def do_GET(self):
        p = self._caminho()
        if p == "/json":
            return eco_json(self)
        if p == "/falha":
            # falha com 500 nas primeiras `ate` chamadas, depois responde 200.
            ate = int(parse_qs(urlparse(self.path).query).get("ate", ["2"])[0])
            n = CONTADORES.get("falha", 0) + 1
            CONTADORES["falha"] = n
            if n <= ate:
                return self._resp(f"falhou na tentativa {n}", 500)
            CONTADORES["falha"] = 0
            return self._resp(json.dumps({"tentativa": n}), extra=[("Content-Type", "application/json")])
        if p.startswith("/status/"):
            return self._resp(f"status {p[8:]}", int(p[8:]))
        if p.startswith("/grande"):
            mb = int(parse_qs(urlparse(self.path).query).get("mb", ["50"])[0])
            n = mb * 1024 * 1024
            self.send_response(200)
            self.send_header("Content-Type", "application/octet-stream")
            self.send_header("Content-Length", str(n))
            self.end_headers()
            pedaco = b"x" * 65536
            for _ in range(n // 65536):
                self.wfile.write(pedaco)
                time.sleep(0.006)  # ~8 MB/s: dá para ver a barra andar
            return
        if p == "/download":
            return self._resp("".join(chr(65 + i % 26) for i in range(2000)))
        if p == "/set":
            self._resp("cookie definido", extra=[("Set-Cookie", "sid=abc123; Path=/"), ("Set-Cookie", "tema=escuro; Path=/; Max-Age=3600")])
        elif p == "/apaga":
            self._resp("cookie apagado", extra=[("Set-Cookie", "sid=; Path=/; Max-Age=0")])
        elif p == "/eco":
            self._resp("Cookie: " + self.headers.get("Cookie", "(nenhum)"))
        elif p == "/lento":
            time.sleep(4)
            try:
                self._resp("demorou 4s")
            except Exception:
                pass
        else:
            self._resp("não achei", 404)

    def do_POST(self):
        n = int(self.headers.get("Content-Length", 0))
        if self._caminho() == "/enviar":
            # lê devagar (~16 MB/s), para a barra de envio ter o que mostrar
            lido = 0
            while lido < n:
                lido += len(self.rfile.read(min(65536, n - lido)))
                time.sleep(0.004)
            return self._resp(f"recebi {lido} bytes")
        corpo = self.rfile.read(n)
        if self._caminho() in ("/json",):
            return eco_json(self, corpo)
        if self._caminho() == "/enviar":
            return self._resp(f"recebi {len(corpo)} bytes")
        if self._caminho() != "/upload":
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

    def do_QUERY(self):
        # RFC 10008 §2: sem conteúdo ou sem Content-Type, o servidor rejeita.
        n = int(self.headers.get("Content-Length", 0))
        corpo = self.rfile.read(n)
        if not corpo or not self.headers.get("Content-Type"):
            return self._resp("QUERY sem corpo ou sem Content-Type", 400)
        if self._caminho() != "/json":
            return self._resp("não achei", 404)
        eco_json(self, corpo)

    def log_message(self, *a):
        pass

if __name__ == "__main__":
    porta = int(sys.argv[1]) if len(sys.argv) > 1 else 8096
    ThreadingHTTPServer(("127.0.0.1", porta), H).serve_forever()
