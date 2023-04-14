#include "device_variant.h"
#include "flash.h"

static uint8_t device_variant_color = 0;
static bool device_variant_btconly = false;
static bool device_variant_ok = false;

static void device_variant_0x01(const uint8_t *data) {
  device_variant_color = data[1];
  device_variant_btconly = data[2] == 1;
  device_variant_ok = true;
}

void device_variant_init(void) {
  uint8_t data[FLASH_OTP_BLOCK_SIZE];

  secbool result = flash_otp_read(FLASH_OTP_BLOCK_DEVICE_VARIANT, 0, data,
                                  FLASH_OTP_BLOCK_SIZE);

  if (sectrue == result) {
    switch (data[0]) {
      case 0x01:
        device_variant_0x01(data);
        break;
      default:
        break;
    }
  }
}

uint8_t device_variant_get_color(void) { return device_variant_color; }

bool device_variant_get_btconly(void) { return device_variant_btconly; }

bool device_variant_present(void) { return device_variant_ok; }
