_start:
	addi	sp,sp,-8

	li		a0,20
	li		a1,22

	sw		a0,0(sp)
	sw		a1,4(sp)

	lw		t0,0(sp)
	lw		t1,4(sp)

	addi	sp,sp,8
_show:
	li		a7,1
	mv		a0,t0
	ecall

	mv		a0,t1
	ecall
_exit:
	li		a7,0
	li		a0,0
	ecall
