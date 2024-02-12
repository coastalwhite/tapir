#include "stdbool.h"
#include "stdint.h"

#define GPIO_BASE 0x1A101000
#define GPIO_DIR ((volatile uint32_t *)GPIO_BASE + 0)
#define GPIO_IN  ((volatile uint32_t *)GPIO_BASE + 4)
#define GPIO_OUT ((volatile uint32_t *)GPIO_BASE + 8)

int main();
void delay();

// DIR
static inline void set_t2_led(bool is_on) {
    uint32_t mask;
    if (is_on) {
        mask = 0b10;
    } else {
        mask = 0;
    }

    (*GPIO_DIR) = mask;
}
// OUT
static inline void set_t4_led(bool is_on) {
    uint32_t mask;
    if (is_on) {
        mask = 0b10;
    } else {
        mask = 0;
    }

    (*GPIO_OUT) = mask;
}
static inline void set_tio(bool tio1, bool tio2) {
    uint32_t mask = 0;
    if (tio1) {
        mask = 0b10000;
    }
    if (tio2) {
        mask |= 0b100000;
    }

    mask = mask & ((*GPIO_OUT) & 0x00000030);
    (*GPIO_OUT) = mask;
}

void delay() {
    volatile int i = 0;
    for (; i < 100; i++)
        asm volatile("nop");
}

int main() {
    set_t2_led(true);
    set_t4_led(false);

    delay();

    unsigned char *start = (unsigned char*) 0;
    unsigned char *end = (unsigned char*) 0x100;

    bool sw_led = false;

    while (start < end) {
        unsigned char b = *start;
        for (int i = 0; i < 8; i++) {
            set_tio(sw_led, (b & 1) == 1);

            b >>= 1;
            sw_led = !sw_led;
            set_t4_led(sw_led);
            delay();
        }

        start += 1;
    }

    set_t2_led(false);

    while (true)
        ;
}