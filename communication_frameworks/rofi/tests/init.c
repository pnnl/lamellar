#include <unistd.h>

#include <rofi.h>
#include "utils.h"

int main(void)
{
  rofi_banner("Init Test");
  rofi_init();

  rofi_verify(0);
  rofi_finit();

  return 0;
}
