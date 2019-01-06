#define EL0 0b00
#define EL1 0b01
#define EL2 0b10
#define EL3 0b11

.section .text.init

.global _start
_start:
    // read cpu affinity, start core 0, halt rest
    mrs     x1, MPIDR_EL1
    and     x1, x1, #3
    cbz     x1, setup

halt:
    // core affinity != 0, halt it
    wfe
    b       halt

setup:
    // store the desired EL1 stack pointer in x1
    adr     x1, _start

    // read the current exception level into x0 (ref: C5.2.1)
    mrs     x0, CurrentEL
    and     x0, x0, #0b1100
    lsr     x0, x0, #2

switch_to_el2:
    // switch to EL2 if we're in EL3. otherwise switch to EL1
    cmp     x0, EL3
    bne     switch_to_el1

    // set-up SCR_EL3 (bits 0, 4, 5, 7, 8, 10) (A53: 4.3.42)
    mov     x2, #0x5b1
    msr     SCR_EL3, x2

    // set-up SPSR and PL switch! (bits 0, 3, 6, 7, 8, 9) (ref: C5.2.20)
    mov     x2, #0x3c9
    msr     SPSR_EL3, x2
    adr     x2, switch_to_el1
    msr     ELR_EL3, x2
    eret

switch_to_el1:
    // switch to EL1 if we're not already in EL1. otherwise continue with start
    cmp     x0, EL1
    beq     set_stack

    // set the stack-pointer for EL1
    msr     SP_EL1, x1

    // enable CNTP for EL1/EL0 (ref: D7.5.2, D7.5.13)
    // NOTE: This doesn't actually enable the counter stream.
    mrs     x0, CNTHCTL_EL2
    orr     x0, x0, #0b11
    msr     CNTHCTL_EL2, x0
    msr     CNTVOFF_EL2, xzr

    // enable AArch64 in EL1 (A53: 4.3.36)
    mov     x0, #(1 << 31)      // Enable AArch64 for EL1
    orr     x0, x0, #(1 << 1)   // RES1 on A-53
    msr     HCR_EL2, x0
    mrs     x0, HCR_EL2

    // enable floating point and SVE (SIMD) (A53: 4.3.38, 4.3.34)
    msr     CPTR_EL2, xzr     // don't trap accessing SVE registers
    mrs     x0, CPACR_EL1
    orr     x0, x0, #(0b11 << 20)
    msr     CPACR_EL1, x0

    // Set SCTLR to known state (RES1: 11, 20, 22, 23, 28, 29) (A53: 4.3.30)
    mov     x2, #0x0800
    movk    x2, #0x30d0, lsl #16
    msr     SCTLR_EL1, x2

    // set up exception handlers
    // FIXME: load `_vectors` addr into appropriate register (guide: 10.4)
    adr     x2, _vectors
    msr     VBAR_EL1, x2

    // change execution level to EL1 (ref: C5.2.19)
    mov     x2, #0x3c5
    msr     SPSR_EL2, x2
    // FIXME: Return to EL1 at `set_stack`.
    adr     x2, set_stack
    msr     ELR_EL2, x2
    eret

set_stack:
    // set the current stack pointer
    mov     sp, x1

zero_bss:
    // load the start address and number of bytes in BSS section
    ldr     x1, =__bss_start
    ldr     x2, =__bss_length

zero_bss_loop:
    // zero out the BSS section, 64-bits at a time
    cbz     x2, go_kmain
    str     xzr, [x1], #8
    sub     x2, x2, #8
    cbnz    x2, zero_bss_loop

go_kmain:
    // jump to kmain, which shouldn't return. halt if it does
    bl      kmain
    b       halt

context_save:
    // FIXME: Save the remaining context to the stack.
    // naive way
    stp     x29, lr, [SP, #-16]! // put current lr in reserved
    stp     x27, x28, [SP, #-16]!
    stp     x25, x26, [SP, #-16]!
    stp     x23, x24, [SP, #-16]!
    stp     x21, x22, [SP, #-16]!
    stp     x19, x20, [SP, #-16]!
    stp     x17, x18, [SP, #-16]!
    stp     x15, x16, [SP, #-16]!
    stp     x13, x14, [SP, #-16]!
    stp     x11, x12, [SP, #-16]!
    stp     x9, x10, [SP, #-16]!
    stp     x7, x8, [SP, #-16]!
    stp     x5, x6, [SP, #-16]!
    stp     x3, x4, [SP, #-16]!
    stp     x1, x2, [SP, #-16]!

    stp     q30, q31, [SP, #-32]!
    stp     q28, q29, [SP, #-32]!
    stp     q26, q27, [SP, #-32]!
    stp     q24, q25, [SP, #-32]!
    stp     q22, q23, [SP, #-32]!
    stp     q20, q21, [SP, #-32]!
    stp     q18, q19, [SP, #-32]!
    stp     q16, q17, [SP, #-32]!
    stp     q14, q15, [SP, #-32]!
    stp     q12, q13, [SP, #-32]!
    stp     q10, q11, [SP, #-32]!
    stp     q8, q9, [SP, #-32]!
    stp     q6, q7, [SP, #-32]!
    stp     q4, q5, [SP, #-32]!
    stp     q2, q3, [SP, #-32]!
    stp     q0, q1, [SP, #-32]!

    mrs x2, TPIDR_EL0
    mrs x1, SP_EL0
    stp     x1, x2, [SP, #-16]!
    mrs x2, SPSR_EL1
    mrs x1, ELR_EL1
    stp     x1, x2, [SP, #-16]!

    mrs x1, ESR_EL1 // esr
    mov x2, sp // tf is sp

    mov x28, lr // save lr to a callee-saved register
    bl handle_exception

    mov lr, x28 // get current lr from r28.

.global context_restore
context_restore:
    // FIXME: Restore the context from the stack.
    ldp     x1, x2, [SP], #16
    msr ELR_EL1, x1
    msr SPSR_EL1, x2
    ldp     x1, x2, [SP], #16
    msr SP_EL0, x1
    msr TPIDR_EL0, x2

    ldp     q0, q1, [SP], #32
    ldp     q2, q3, [SP], #32
    ldp     q4, q5, [SP], #32
    ldp     q6, q7, [SP], #32
    ldp     q8, q9, [SP], #32
    ldp     q10, q11, [SP], #32
    ldp     q12, q13, [SP], #32
    ldp     q14, q15, [SP], #32
    ldp     q16, q17, [SP], #32
    ldp     q18, q19, [SP], #32
    ldp     q20, q21, [SP], #32
    ldp     q22, q23, [SP], #32
    ldp     q24, q25, [SP], #32
    ldp     q26, q27, [SP], #32
    ldp     q28, q29, [SP], #32
    ldp     q30, q31, [SP], #32

    ldp     x1, x2, [SP], #16
    ldp     x3, x4, [SP], #16
    ldp     x5, x6, [SP], #16
    ldp     x7, x8, [SP], #16
    ldp     x9, x10, [SP], #16
    ldp     x11, x12, [SP], #16
    ldp     x13, x14, [SP], #16
    ldp     x15, x16, [SP], #16
    ldp     x17, x18, [SP], #16
    ldp     x19, x20, [SP], #16
    ldp     x21, x22, [SP], #16
    ldp     x23, x24, [SP], #16
    ldp     x25, x26, [SP], #16
    ldp     x27, x28, [SP], #16
    ldr     x29, [SP], #16
    ret

#define HANDLER(source, kind) \
    .align 7; \
    stp     lr, x0, [SP, #-16]!; \
    mov     x0, ##source; \
    movk    x0, ##kind, LSL #16; \
    bl      context_save; \
    ldp     lr, x0, [SP], #16; \
    eret

.align 11
_vectors:
    // FIXME: Setup the 16 exception vectors.
    HANDLER(0,0)
    HANDLER(0,1)
    HANDLER(0,2)
    HANDLER(0,3)
    HANDLER(1,0)
    HANDLER(1,1)
    HANDLER(1,2)
    HANDLER(1,3)
    HANDLER(2,0)
    HANDLER(2,1)
    HANDLER(2,2)
    HANDLER(2,3)
    HANDLER(3,0)
    HANDLER(3,1)
    HANDLER(3,2)
    HANDLER(3,3)
