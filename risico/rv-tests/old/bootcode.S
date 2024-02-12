.globl main
main:
	# Move stack pointer to proper position
	li		sp,0
	lui		sp,0x00108

read_range_start:
	jal    	ra,usb_read
	mv		t0,a0

    ebreak

	jal    	ra,usb_read
	mv		t1,a0

copy_word_start:
	beq		t0,t1,post_copy

	jal    	ra,usb_read
	sw		t0,0(a0)

	addi	t0,t0,4
	beq		t0,t1,read_range_start
	j		copy_word_start

post_copy:
	li		t0,0

write_loop_start:
	li		t1,16
	beq 	t0,t1,end

	lw		a0,0(t0)
    auipc   ra,0x4
	j     	usb_write
	
	addi	t0,t0,4
	j		write_loop_start

end:
	j		post_copy

# GPIO_DIR  = 0x1A10_1000
# GPIO_IN   = 0x1A10_1004
# GPIO_OUT  = 0x1A10_1008
#
# GPIO_IN
# [9]    = active
# [8]    = turn (1 = PULPINO, 0 = IO)
# [7:0]  = data_out
#
# uint32_t usb_read() {
#     while ((*GPIO_IN) & 0x200 == 0) ;
#
#     uint32_t out = 0;
#     for (int i = 0; i < 4; i++) {
#         while ((*GPIO_IN) & 0x100 == 0) ;
#
#         out <<= 8;
#         out |= (*GPIO_IN) & 0xFF;

#		  uint32_t mask = 0x100;
#		  if (i == 3) {
#     	      mask |= 0x200;
# 		  }
#         (*GPIO_IN) ^= mask;
#     }
#
#	  return out;
# }
#

# Read a word from the USB into `a0`
.globl usb_read
usb_read:
	addi 	sp,sp,-16
	sw		a1,0(sp)
	sw		a2,4(sp)
	sw		t0,8(sp)
	sw		t1,12(sp)

	# t0 = GPIO_IN
	lui 	t0,0x1A101

	# while ((*GPIO_IN) & 0x200 == 0) ;
usb_read_wait_for_usb_active:
	lw		a1,4(t0)
	andi	a1,a1,0x200
    ebreak
	beqz	a1,usb_read_wait_for_usb_active
	
	li		a0,0 # out = 0
	li		a2,4 # i = 4

usb_read_read_byte_loop_begin:
	beqz	a2,usb_read_read_finished

	# while ((*GPIO_IN) & 0x100 == 0) ;
usb_read_wait_for_turn:
	lw		a1,4(t0)
	li		t1,0x100
	and		a1,a1,t1
	beqz	a1,usb_read_wait_for_turn

	# out <<= 8
	sll		a0,a0,8 

	# out |= (*GPIO_IN) & 0xFF
	li		t1,0xFF  
	lw		a1,4(t0)
	and		a1,a1,t1
	or		a0,a0,a1

	# (*GPIO_IN) ^= (i == 0) ? 0x300 : 0x100;
	li		t1,0x100  

	addi	a1,a2,-1
	bnez	a1,usb_read_hand_over_turn
	li		t1,0x300
	
usb_read_hand_over_turn:
	lw		a1,4(t0)
	xor		a1,a1,t1
	sw		a1,4(t0)

	addi	a2,a2,-1 # i -= 1
	j		usb_read_read_byte_loop_begin

usb_read_read_finished:

	lw		a1,0(sp)
	lw		a2,4(sp)
	lw		t0,8(sp)
	lw		t1,12(sp)
	addi	sp,sp,16

	ret

# GPIO_DIR
# [10]  = read by host
# [9]   = active
# [8]   = turn (1 = PULPINO, 0 = IO)
# [7:0] = data_out
#
# void usb_write(uint32_t data) {
#     (*GPIO_DIR) = 0x300
#
#     for (int i = 0; i < 4; i++) {
#		  (*GPIO_DIR) |= data & 0xFF;
#		  data >>= 8;
#
#         (*GPIO_DIR) ^= 0x100;
#         while ((*GPIO_DIR) & 0x100 == 0) ;
#     }
#
#     (*GPIO_DIR) = 0x000;
# }
#
# Write a word from `a0` to the USB
.globl usb_write
usb_write:
	addi 	sp,sp,-20
	sw		a0,0(sp)
	sw		a1,4(sp)
	sw		a2,8(sp)
	sw		t0,12(sp)
	sw		t1,16(sp)

	# t0 = GPIO_DIR
	li		t0,0
	lui 	t0,0x1A101

	# (*GPIO_DIR) = 0x300
	li		a1,0x300
	sw		a1,0(t0)
	
	li		a2,4 # i = 4

usb_write_write_byte_loop_begin:
	beqz	a2,usb_write_write_finished

	# (*GPIO_DIR) |= data & 0xFF;
	li		t1,0xFF
	and		a0,a0,t1
	lw		a1,0(t0)
	or		a1,a0,a1
	sw		a1,0(t0)

	# data >>= 8;
	srl		a0,a0,8

	# (*GPIO_DIR) ^= 0x100;
	li		t1,0x100
	lw		a1,0(t0)
	xor		a1,t0,a1
	sw		a1,0(t0)

	# while ((*GPIO_DIR) & 0x100 == 0) ;
usb_write_wait_for_turn:
	lw		a1,0(t0)
	li		t1,0x100
	and		a1,a1,t1
	beqz	a1,usb_write_wait_for_turn

	addi	a2,a2,-1 # i -= 1
	j		usb_write_write_byte_loop_begin

usb_write_write_finished:

	# (*GPIO_DIR) = 0x000;
	sw		zero,0(t0)

	lw		a0,0(sp)
	lw		a1,4(sp)
	lw		a2,8(sp)
	lw		t0,12(sp)
	lw		t1,16(sp)
	addi	sp,sp,20

	ret

