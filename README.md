# uniedge

Ponto de partida para um unikernel em Rust, usando:

- `no_std` e `no_main`
- target bare metal `aarch64-unknown-none-softfloat`
- QEMU `virt` como ambiente de execução
- UART PL011 para saída serial
- `virtio-net` + `smoltcp` para servir HTTP em `:8080`

Escolhi `aarch64` no QEMU porque o bootstrap inicial é muito menor do que em `x86_64`, o que permite começar com uma base realmente mínima e controlável. Depois disso, fica simples evoluir para memória, interrupções, allocator, timer, rede e, se fizer sentido, portar para outro alvo.

## Requisitos

- Rust `stable`
- target `aarch64-unknown-none-softfloat`
- `qemu-system-aarch64`

## Setup

```bash
rustup target add aarch64-unknown-none-softfloat
```

## Executar

```bash
cargo run
```

Você deve ver algo como:

```text
UniEdge is alive!
serial: pl011 | web: http://127.0.0.1:8080
```

Para sair do QEMU em modo texto, use `Ctrl+A` seguido de `X`.

Em outro terminal, valide a página:

```bash
curl http://127.0.0.1:8080
```

Resposta esperada:

```text
UniEdge is alive!
```

## Estrutura

- `src/boot.S`: bootstrap inicial e limpeza de `.bss`
- `src/main.rs`: entrypoint Rust, bootstrap do runtime e panic handler
- `src/net.rs`: `virtio-net`, `smoltcp` e servidor HTTP
- `src/bootfx.rs`: efeito visual de inicialização no terminal serial
- `src/time.rs`: timer monotônico via generic timer do ARM
- `src/allocator.rs`: heap local para as crates `alloc`
- `src/platform.rs`: descoberta do transporte `virtio-mmio` via DTB
- `linker.ld`: layout do binário e stack de boot
- `scripts/run-qemu.sh`: runner usado pelo `cargo run`
- `.cargo/config.toml`: target, linker e runner

## Próximos passos naturais

- inicializar exception vectors
- adicionar timer e interrupções de rede
- implementar mapeamento de memória
- trocar IP estático por DHCP
- expor um pequeno runtime para serviços do unikernel
