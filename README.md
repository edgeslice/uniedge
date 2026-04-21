# uniedge

Ponto de partida para um unikernel em Rust, usando:

- `no_std` e `no_main`
- target bare metal `aarch64-unknown-none-softfloat`
- QEMU `virt` como ambiente de execução
- UART PL011 para saída serial

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
uniEdge kernel booted
target: qemu-system-aarch64 virt
status: no_std / no_main / bare metal
```

Para sair do QEMU em modo texto, use `Ctrl+A` seguido de `X`.

## Estrutura

- `src/boot.S`: bootstrap inicial e limpeza de `.bss`
- `src/main.rs`: entrypoint Rust, UART e panic handler
- `linker.ld`: layout do binário e stack de boot
- `scripts/run-qemu.sh`: runner usado pelo `cargo run`
- `.cargo/config.toml`: target, linker e runner

## Próximos passos naturais

- inicializar exception vectors
- adicionar timer e interrupções
- definir allocator
- implementar mapeamento de memória
- expor um pequeno runtime para serviços do unikernel
