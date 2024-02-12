_start:
    lui     t0,0x100
    lw      zero,0(t0)
    sw      zero,0(t0)
end:
    ebreak
