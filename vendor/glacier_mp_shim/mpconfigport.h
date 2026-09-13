// Configuração do MicroPython para o "port" embutido no glacier-ui.
//
// Espelha examples/embedding/mpconfigport.h do upstream (mesma configuração
// mínima, ROM level MINIMUM) — é a mesma combinação já usada para gerar
// vendor/micropython_embed/genhdr/*.h (moduledefs.h, qstrdefs.generated.h
// etc.), que dependem exatamente destas macros. Mudar algo aqui exige
// regenerar esse diretório (rodar `make -f micropython_embed.mk` numa árvore
// do MicroPython na mesma tag — ver vendor/micropython_embed/LICENSE para a
// versão vendorizada).
#include <port/mpconfigport_common.h>

#define MICROPY_CONFIG_ROM_LEVEL (MICROPY_CONFIG_ROM_LEVEL_MINIMUM)

#define MICROPY_ENABLE_COMPILER (1)
#define MICROPY_ENABLE_GC (1)
#define MICROPY_PY_GC (1)
#define MICROPY_PY_SYS (0)

// `__getattr__`/`__setattr__` num objeto Python — o que dá ao `ctx` a
// sintaxe `ctx.chave` (dotdict), além de `ctx["chave"]`. Fora do ROM level
// mínimo por padrão; ligar só esta macro evita puxar o resto das features de
// "EXTRA_FEATURES" junto. Muda o conjunto de qstrs que o genhdr precisa
// conhecer — quem mexer aqui precisa regenerar
// vendor/micropython_embed/genhdr (rodar `make -f micropython_embed.mk` numa
// árvore do MicroPython na mesma tag, com este define, e recopiar o
// diretório gerado).
#define MICROPY_PY_DELATTR_SETATTR (1)
