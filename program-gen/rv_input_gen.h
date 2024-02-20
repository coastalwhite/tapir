#ifndef __RV_INPUT_GEN_H__
#define __RV_INPUT_GEN_H__

#include <stdint.h>

extern "C" {
	extern uint8_t* rv_generate_instructions(uint32_t entry, uint32_t *length);
	extern void rv_free_instructions(uint8_t *ptr, uint32_t length);
}

#endif // __RV_INPUT_GEN_H__