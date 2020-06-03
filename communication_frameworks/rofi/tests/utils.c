#include <stdio.h>
#include <time.h>
#include <config.h>

#include "utils.h"

//#define VERBOSE

struct timespec start, end;

void rofi_banner(char* name)
{
	time_t t;

	printf("\n\n");
	printf("===========================================\n");
	printf("    %s\n", PACKAGE_NAME);
	printf("    %s\n",name);
	printf("===========================================\n");
	printf("Version:    %s\n", PACKAGE_VERSION);
	t = time(NULL);
	printf("Start time: %s",ctime(&t));
	printf("-------------------------------------------\n");
	clock_gettime(CLOCK_MONOTONIC, &start);
}

void rofi_verify(int res)
{
	time_t t;

	clock_gettime(CLOCK_MONOTONIC, &end);
	t = time(NULL);
	printf("-------------------------------------------\n");
	printf("End time:       %s", ctime(&t));
	printf("Result:         %s\n",res? TEST_FAILED: TEST_SUCCESS);
	printf("Execution time: %f seconds\n", ((float)tdiff(end,start))/BILLION);
	printf("===========================================\n");
}
