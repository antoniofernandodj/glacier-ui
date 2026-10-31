#!/usr/bin/env python3
"""Gera os áudios do exemplo `gvb_audio`, sem encoder nem arquivo de terceiros.

  python3 tools/gera_audio_exemplo.py examples/gvb/audio/sons

- `tom.wav`: seno de 440 Hz com decaimento, 8 s, 16 bits mono 22,05 kHz (audível).
- `silencio.mp3`: 8 s de MP3 válido (MPEG-1 Layer III, 128 kbps, 44,1 kHz, estéreo) feito só de
  quadros de silêncio, com o cabeçalho `Info` (contagem de quadros) que dá a duração ao
  decodificador. Serve para exercitar o caminho de decodificação de MP3, não para ouvir.
"""
import math, struct, sys, wave, os

saida = sys.argv[1] if len(sys.argv) > 1 else "."
os.makedirs(saida, exist_ok=True)

# WAV
taxa, seg = 22050, 8
w = wave.open(os.path.join(saida, "tom.wav"), "wb")
w.setnchannels(1); w.setsampwidth(2); w.setframerate(taxa)
quadros = bytearray()
for i in range(taxa * seg):
    t = i / taxa
    amp = 0.25 * (1 - 0.6 * (t / seg))
    quadros += struct.pack("<h", int(32767 * amp * math.sin(2 * math.pi * 440 * t)))
w.writeframes(bytes(quadros)); w.close()

# MP3: quadros de 417 bytes (144 * 128000 / 44100), 1152 amostras cada.
TAM = 417
n = math.ceil(seg * 44100 / 1152)
cab = bytes([0xFF, 0xFB, 0x90, 0x00])       # MPEG1 L3, sem CRC, 128 kbps, 44,1 kHz, estéreo
primeiro = bytearray(cab + bytes(TAM - 4))
# Cabeçalho `Info` (CBR) logo depois das informações laterais (32 bytes no estéreo MPEG-1).
primeiro[36:40] = b"Info"
primeiro[40:44] = struct.pack(">I", 0x0001)  # flag: o campo de quadros existe
primeiro[44:48] = struct.pack(">I", n)
with open(os.path.join(saida, "silencio.mp3"), "wb") as f:
    f.write(primeiro)
    for _ in range(n - 1):
        f.write(cab + bytes(TAM - 4))
print("gerado em", saida)
