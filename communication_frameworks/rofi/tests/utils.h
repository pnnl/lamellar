#ifndef ROFI_TEST_UTILS_H
#define ROFI_TEST_UTILS_H

#include <fcntl.h>
#include <unistd.h>
#include <stdlib.h>

#define KNRM  "\x1B[0m"
#define KRED  "\x1B[31m"
#define KGRN  "\x1B[32m"
#define KYEL  "\x1B[33m"
#define KBLU  "\x1B[34m"
#define KMAG  "\x1B[35m"
#define KCYN  "\x1B[36m"
#define KWHT  "\x1B[37m"

#define TEST_SUCCESS "\x1B[32mSUCCESS \x1B[0m"
#define TEST_FAILED   "\x1B[31mFAILED \x1B[0m"

#define BILLION 1000000000ULL
#define MILLION 1000000ULL
#define tdiff(end,start) BILLION * (end.tv_sec - start.tv_sec) + end.tv_nsec - start.tv_nsec

void rofi_banner(char*);
void rofi_verify(int);
#endif
