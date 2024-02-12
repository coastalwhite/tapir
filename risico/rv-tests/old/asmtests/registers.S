_start:
    li      x1,43
    li      x2,44
    li      x3,45
    li      x4,46
    li      x5,47
    li      x6,48
    li      x7,49
    li      x8,50
    li      x9,51
    li      x10,52
    li      x11,53
    li      x12,54
    li      x13,55
    li      x14,56
    li      x15,57
    li      x16,58
    li      x17,59
    li      x18,60
    li      x19,61
    li      x20,62
    li      x21,63
    li      x22,64
    li      x23,65
    li      x24,66
    li      x25,67
    li      x26,68
    li      x27,69
    li      x28,70
    li      x29,71
    li      x30,72
    li      x31,73

    sw      x0,0(zero)
    sw      x1,4(zero)
    sw      x2,8(zero)
    sw      x3,12(zero)
    sw      x4,16(zero)
    sw      x5,20(zero)
    sw      x6,24(zero)
    sw      x7,28(zero)
    sw      x8,32(zero)
    sw      x9,36(zero)
    sw      x10,40(zero)
    sw      x11,44(zero)
    sw      x12,48(zero)
    sw      x13,52(zero)
    sw      x14,56(zero)
    sw      x15,60(zero)
    sw      x16,64(zero)
    sw      x17,68(zero)
    sw      x18,72(zero)
    sw      x19,76(zero)
    sw      x20,80(zero)
    sw      x21,84(zero)
    sw      x22,88(zero)
    sw      x23,92(zero)
    sw      x24,96(zero)
    sw      x25,100(zero)
    sw      x26,104(zero)
    sw      x27,108(zero)
    sw      x28,112(zero)
    sw      x29,116(zero)
    sw      x30,120(zero)
    sw      x31,124(zero)

    li      a7,1

    lw      a0,0(zero)
    ecall

    lw      a0,4(zero)
    ecall

    lw      a0,8(zero)
    ecall

    lw      a0,12(zero)
    ecall

    lw      a0,16(zero)
    ecall

    lw      a0,20(zero)
    ecall

    lw      a0,24(zero)
    ecall

    lw      a0,28(zero)
    ecall

    lw      a0,32(zero)
    ecall

    lw      a0,36(zero)
    ecall

    lw      a0,40(zero)
    ecall

    lw      a0,44(zero)
    ecall

    lw      a0,48(zero)
    ecall

    lw      a0,52(zero)
    ecall

    lw      a0,56(zero)
    ecall

    lw      a0,60(zero)
    ecall

    lw      a0,64(zero)
    ecall

    lw      a0,68(zero)
    ecall

    lw      a0,72(zero)
    ecall

    lw      a0,76(zero)
    ecall

    lw      a0,80(zero)
    ecall

    lw      a0,84(zero)
    ecall

    lw      a0,88(zero)
    ecall

    lw      a0,92(zero)
    ecall

    lw      a0,96(zero)
    ecall

    lw      a0,100(zero)
    ecall

    lw      a0,104(zero)
    ecall

    lw      a0,108(zero)
    ecall

    lw      a0,112(zero)
    ecall

    lw      a0,116(zero)
    ecall

    lw      a0,120(zero)
    ecall

    lw      a0,124(zero)
    ecall

_end:
    li      a7,0
    li      a0,0
    ecall