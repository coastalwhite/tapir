_start:
    li      a0,20
    li      a1,22
    add     a0,a0,a1
    
    li      a7,1
    ecall

    li      a7,3
    addi    a0,a0,-42
    ecall

end:
    li      a7,0
    ecall

