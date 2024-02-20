#include <unistd.h>

#include "utils.h"
#include <rofi.h>

int main(void) {
    rofi_banner("Init Test");
    rofi_init(NULL, "ib0");
    // rofi_init(NULL, NULL);

    rofi_verify(0);
    rofi_finit();

    return 0;
}
