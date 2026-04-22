# uniedge

`uniedge` é um ponto de partida para estudar e construir um unikernel em Rust.
Hoje ele sobe direto no QEMU, sem Linux, sem `libstd` e sem userspace, inicializa um runtime mínimo e expõe um servidor HTTP muito pequeno em `:8080`.

O foco do projeto é didático: partir de uma base realmente bare metal e ir adicionando, de forma controlada, memória, tempo, rede e serviços.

## O que é um unikernel

Um unikernel é uma imagem única que empacota:

- a aplicação
- apenas as partes do sistema operacional necessárias para essa aplicação
- o código de boot para rodar diretamente sobre um hipervisor ou hardware compatível

Na prática, em vez de subir um sistema operacional geral e depois executar um processo dentro dele, o serviço já nasce como o próprio sistema.

Comparação rápida:

- container: compartilha o kernel do host
- VM tradicional: sobe um SO completo dentro da máquina virtual e executa aplicações dentro dele
- unikernel: compila aplicação + runtime de baixo nível em uma única imagem especializada

Vantagens típicas:

- menor superfície de código
- boot mais rápido
- maior controle sobre o ambiente
- menos camadas entre o serviço e o hardware virtual

Trade-offs:

- menos ferramentas prontas de observabilidade e debug
- menos abstrações de sistema operacional
- suporte a drivers e plataforma costuma ser mais manual
- geralmente é um ambiente de propósito único, não um SO geral

Neste repositório, o serviço embutido no unikernel é um servidor HTTP mínimo que retorna `UniEdge is alive!`.

## O que o projeto faz hoje

- usa `no_std` e `no_main`
- compila para `aarch64-unknown-none-softfloat`
- inicia no QEMU `virt`
- faz bootstrap inicial em assembly
- limpa `.bss` e transfere o controle para `kmain()`
- inicializa um heap local para uso de `alloc`
- escreve logs e feedback visual via UART PL011
- descobre um dispositivo `virtio-net` por DTB ou varredura MMIO
- configura uma pilha IPv4 com `smoltcp`
- responde HTTP em `:8080`

Escolher `aarch64` no QEMU simplifica bastante o primeiro bootstrap em comparação com `x86_64`, o que ajuda a manter o projeto pequeno, explícito e fácil de evoluir.

## Arquitetura em alto nível

Fluxo de boot:

`boot.S` -> stack inicial -> limpeza de `.bss` -> `kmain()` -> allocator -> bootfx/serial -> descoberta `virtio-mmio` -> rede -> loop HTTP

Fluxo de rede:

- o guest usa o IP estático `10.0.2.15/24`
- o QEMU faz `hostfwd` de `127.0.0.1:8080` no host para `:8080` dentro da VM
- o servidor HTTP escuta uma conexão TCP por vez e responde um texto plano curto

## Requisitos

- Rust `stable`
- target `aarch64-unknown-none-softfloat`
- `qemu-system-aarch64`

## Setup

```bash
rustup target add aarch64-unknown-none-softfloat
```

## Como executar

O runner já está configurado em `.cargo/config.toml`, então `cargo run` compila e sobe o QEMU:

```bash
cargo run
```

Também existem atalhos no `Makefile`:

```bash
make run
make run-release
```

Entre as linhas exibidas na serial, você deve ver algo como:

```text
UniEdge is alive!
serial: pl011 | web: http://127.0.0.1:8080
virtio-net mmio @ 0x000000000a000000 (512 bytes)
```

Em outro terminal:

```bash
curl http://127.0.0.1:8080
```

Resposta esperada:

```text
UniEdge is alive!
```

Para sair do QEMU em modo texto, use `Ctrl+A` seguido de `X`.

## Estrutura do código

- `src/boot.S`: bootstrap inicial, stack de boot e limpeza de `.bss`
- `src/main.rs`: entrypoint Rust, inicialização do runtime e `panic_handler`
- `src/console.rs`: driver mínimo da UART PL011 para saída serial
- `src/bootfx.rs`: animação de boot exibida na serial
- `src/allocator.rs`: heap local para permitir uso de `alloc`
- `src/time.rs`: leitura do generic timer do ARM e delays simples
- `src/platform.rs`: descoberta de dispositivos `virtio-mmio` via DTB e fallback por varredura MMIO
- `src/net.rs`: integração `virtio-net` + `smoltcp` + servidor HTTP
- `linker.ld`: layout do binário e símbolos usados no bootstrap
- `scripts/run-qemu.sh`: comando do QEMU usado como runner do target
- `.cargo/config.toml`: target padrão, linker e runner

## Limitações atuais

O projeto ainda é uma base mínima. Hoje ele não tenta ser um unikernel "completo":

- não há MMU, paginação ou isolamento de memória
- não há exception vectors nem interrupções configuradas
- a rede opera por polling em loop ativo
- o IP é estático
- há apenas um serviço HTTP simples
- o ambiente está acoplado ao QEMU `virt` e ao dispositivo `virtio-net`

## Próximos passos naturais

- inicializar exception vectors
- adicionar timer e interrupções de rede
- implementar mapeamento de memória
- trocar IP estático por DHCP
- expor um pequeno runtime para serviços do unikernel
