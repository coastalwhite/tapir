#: SYSCALL=linux
_start:	
	# Exit Syscall
	li		a7,93
	li		a0,0
	ecall	