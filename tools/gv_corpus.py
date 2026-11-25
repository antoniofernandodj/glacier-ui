#!/usr/bin/env python3
"""Conta, num diretório de `.gv`/`.gss`, as formas que o
`docs/PLANO_LINGUAGEM_GV_GSS.md` deprecia ou substitui.

É a régua da migração: rodado sobre o `rustploy-gui/views` antes e depois de
cada fase, mostra o que ainda resta. Quando uma linha chega a zero, a forma velha
pode sair do motor.

    python3 tools/gv_corpus.py ../rustploy/rustploy-gui/views
"""

import re
import sys
from pathlib import Path

GV = {
    "atributo equals/eq": r"\b(?:equals|eq)\s*=",
    "atributo not_equals/ne": r"\b(?:not_equals|ne)\s*=",
    "atributo one_of/equals_any": r"\b(?:one_of|equals_any)\s*=",
    "atributo contains/has": r"\b(?:contains|has)\s*=",
    "atributo matches": r"\bmatches\s*=",
    "atributo empty/not_empty": r"\b(?:empty|not_empty)\s*=",
    "template(if/else …)": r"\btemplate\s*\(\s*(?:if|else)",
    "template(:for_each …)": r"\btemplate\s*\([^)]*:for_each",
    ":for_each num nó": r":for_each\s*=",
    "comparação com \"true\"/\"false\"": r"[=!]=\s*\"(?:true|false)\"",
    "comparação com \"0\"": r"[=!]=\s*\"0\"",
    "ação handler:arg": r"\bon_\w+\s*=\s*[A-Za-z_][\w]*:[^\s,)]+",
    "i18n \"{t('…')}\" sozinho": r"\"\{t\('[^']*'\)\}\"",
    "disabled = @busy_*": r"disabled\s*=\s*@busy_",
    "else como atributo": r"[(,]\s*else\s*[,)]",
}

GSS = {
    "hidden:": r"\bhidden\s*:",
    "text_color:": r"\btext_color\s*:",
    "bold:": r"\bbold\s*:",
}


def conta(arquivos, padroes):
    total = {k: 0 for k in padroes}
    for arq in arquivos:
        texto = arq.read_text(encoding="utf-8")
        for nome, rx in padroes.items():
            total[nome] += len(re.findall(rx, texto))
    return total


def main():
    if len(sys.argv) != 2:
        print(__doc__)
        sys.exit(2)
    raiz = Path(sys.argv[1])
    gv = sorted(raiz.rglob("*.gv"))
    gss = sorted(raiz.rglob("*.gss"))
    print(f"{len(gv)} arquivos .gv, {len(gss)} .gss em {raiz}\n")
    print("— .gv")
    for nome, n in conta(gv, GV).items():
        print(f"  {n:5d}  {nome}")
    print("— .gss (inclui os style \"\"\"…\"\"\" dos .gv)")
    for nome, n in conta(gss + gv, GSS).items():
        print(f"  {n:5d}  {nome}")


if __name__ == "__main__":
    main()
