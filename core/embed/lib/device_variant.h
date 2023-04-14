#ifndef _DEVICE_VARIANT_H
#define _DEVICE_VARIANT_H

#include <stdbool.h>
#include <stdint.h>

void device_variant_init(void);
bool device_variant_present(void);
uint8_t device_variant_get_color(void);
bool device_variant_get_btconly(void);

#endif  //_DEVICE_VARIANT_H
