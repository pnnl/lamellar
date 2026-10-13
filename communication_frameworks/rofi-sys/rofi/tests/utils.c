#include <stdio.h>
#include <time.h>
#include <config.h>

#include "utils.h"

//#define VERBOSE

struct timespec start, end;

void rofi_banner(char* name)
{
	time_t t;

	fprintf(stderr, "\n\n");
	fprintf(stderr, "===========================================\n");
	fprintf(stderr, "    %s\n", PACKAGE_NAME);
	fprintf(stderr, "    %s\n", name);
	fprintf(stderr, "===========================================\n");
	fprintf(stderr, "Version:    %s\n", PACKAGE_VERSION);
	t = time(NULL);
	fprintf(stderr, "Start time: %s", ctime(&t));
	fprintf(stderr, "-------------------------------------------\n");
	clock_gettime(CLOCK_MONOTONIC, &start);
}

void rofi_verify(int res)
{
	time_t t;

	clock_gettime(CLOCK_MONOTONIC, &end);
	t = time(NULL);
	fprintf(stderr, "-------------------------------------------\n");
	fprintf(stderr, "End time:       %s", ctime(&t));
	fprintf(stderr, "Result:         %s\n", res ? TEST_FAILED : TEST_SUCCESS);
	fprintf(stderr, "Execution time: %f seconds\n", ((float)tdiff(end, start)) / BILLION);
	fprintf(stderr, "===========================================\n");
}
