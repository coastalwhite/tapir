_start:
    lui         a0,0x100
    rdcycle     t0
    lw          zero,0(a0)
    rdcycle     t1
    sub         t0,t1,t0
end:
    li          a7,0
    ecall
