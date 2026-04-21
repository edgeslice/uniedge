.section .text.boot, "ax"
.global _start
.type _start, %function

_start:
    ldr x0, =__boot_stack_top
    mov sp, x0

    ldr x0, =__bss_start
    ldr x1, =__bss_end

0:
    cmp x0, x1
    b.hs 1f
    str xzr, [x0], #8
    b 0b

1:
    bl kmain

2:
    wfe
    b 2b

.size _start, . - _start
