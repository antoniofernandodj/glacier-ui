#!/usr/bin/env python3
"""Injeta mouse e teclado numa janela X11 (XTEST), para conferir o que o Glacier desenha.

Uso (a janela precisa estar sob X11: `WAYLAND_DISPLAY= WGPU_BACKEND=gl cargo run --example X`):

    python3 tools/xtest.py mv:1100,300 click Tab Tab Return
    python3 tools/xtest.py drag:340,225,640,225 wheel:25 shift+Tab ctrl+c

Capturar a janela:  xwd -id <id> | convert xwd:- png:saida.png
(o id sai de `xwininfo -root -tree`). Precisa de python-xlib, xwd e ImageMagick.
Coordenadas são da TELA, não da janela.
"""
import sys, time
from Xlib import display, X, XK
from Xlib.ext import xtest
d = display.Display()
def mover(x, y):
    xtest.fake_input(d, X.MotionNotify, x=x, y=y); d.sync(); time.sleep(0.2)
def clicar():
    xtest.fake_input(d, X.ButtonPress, 1); d.sync(); time.sleep(0.05)
    xtest.fake_input(d, X.ButtonRelease, 1); d.sync(); time.sleep(0.3)
def tecla(spec):
    # "ctrl+c", "shift+Tab", "ctrl+shift+Tab", "Return"
    partes = spec.split("+")
    nome, mods = partes[-1], partes[:-1]
    nomes_mod = {"shift": "Shift_L", "ctrl": "Control_L", "alt": "Alt_L"}
    mk = [d.keysym_to_keycode(XK.string_to_keysym(nomes_mod[m])) for m in mods]
    kc = d.keysym_to_keycode(XK.string_to_keysym(nome))
    for m in mk:
        xtest.fake_input(d, X.KeyPress, m)
    xtest.fake_input(d, X.KeyPress, kc); d.sync(); time.sleep(0.05)
    xtest.fake_input(d, X.KeyRelease, kc)
    for m in reversed(mk):
        xtest.fake_input(d, X.KeyRelease, m)
    d.sync(); time.sleep(0.5)
for a in sys.argv[1:]:
    if a.startswith("mv:"):
        x, y = a[3:].split(","); mover(int(x), int(y))
    elif a == "click": clicar()
    elif a.startswith("drag:"):
        x1, y1, x2, y2 = map(int, a[5:].split(","))
        mover(x1, y1)
        xtest.fake_input(d, X.ButtonPress, 1); d.sync(); time.sleep(0.1)
        for i in range(1, 11):
            mover(x1 + (x2 - x1) * i // 10, y1 + (y2 - y1) * i // 10)
        xtest.fake_input(d, X.ButtonRelease, 1); d.sync(); time.sleep(0.4)
    elif a.startswith("wheel:"):
        for _ in range(int(a[6:])):
            xtest.fake_input(d, X.ButtonPress, 5); d.sync(); xtest.fake_input(d, X.ButtonRelease, 5); d.sync(); time.sleep(0.05)
        time.sleep(0.4)
    else: tecla(a)
