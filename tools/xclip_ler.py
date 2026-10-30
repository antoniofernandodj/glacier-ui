#!/usr/bin/env python3
"""Lê a seleção CLIPBOARD do X11 (o que o dono dela serve), sem xclip/xsel.

    python3 tools/xclip_ler.py          # imprime o texto, ou "(vazio)"
"""
import sys, time
from Xlib import X, display, Xatom

d = display.Display()
w = d.screen().root.create_window(0, 0, 1, 1, 0, d.screen().root_depth)
clip = d.intern_atom("CLIPBOARD")
utf8 = d.intern_atom("UTF8_STRING")
prop = d.intern_atom("XTEST_LER")
dono = d.get_selection_owner(clip)
if dono == X.NONE:
    print("(sem dono: ninguém copiou nada)"); sys.exit(1)
w.convert_selection(clip, utf8, prop, X.CurrentTime)
d.flush()
fim = time.time() + 3
while time.time() < fim:
    if d.pending_events():
        e = d.next_event()
        if e.type == X.SelectionNotify:
            if e.property == X.NONE:
                print("(o dono recusou a conversão)"); sys.exit(1)
            r = w.get_full_property(prop, X.AnyPropertyType)
            print(r.value.decode("utf-8", "replace") if r else "(vazio)")
            sys.exit(0)
    else:
        time.sleep(0.05)
print("(timeout esperando o dono)"); sys.exit(1)
