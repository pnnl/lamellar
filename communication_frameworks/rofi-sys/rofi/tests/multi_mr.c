#include <unistd.h>
#include <stdio.h>
#include <string.h>
#include <time.h>
#include <math.h>

#include <rofi.h>
#include "utils.h"

#define N (1UL<<15)
//#define VERBOSE

typedef struct{
	unsigned long size;
	struct timespec start, end;
	double time;
	double tput;
} results_t;

static inline int verify_data(char* in, char* out, unsigned long size)
{
	unsigned long i;

	for(i=0; i<size; i++){
#ifdef VERBOSE
		printf("%d: %lu\n", i, out[i]);
#endif
		if(in[i] != out[i]){
			printf("ERROR %lu: %c != %c\n", i, in[i], out[i]);
			return -1;
		}
	}

	return 0;
}

int main(void)
{
	unsigned int me, np;
	int ret = 0;
	char* src;
	char* dst;
	unsigned int i;

	rofi_init("verbs",NULL);
	
        np = rofi_get_size();
	if(np != 2){
		printf("Invalid number of processes (%u) (Required 2)! Aborting.\n", np);
		goto out;
	}

	me = rofi_get_id();

	if(!me)
		rofi_banner("ROFI Multi RMA test");

	ret = rofi_alloc(N, 0x0, (void**) &src);
	if(ret){
		printf("Error allocating source RMA memory. Aborting.\n");
		goto out;
	}

	ret = rofi_alloc(N, 0x0, (void**) &dst);
	if(ret){
		printf("Error allocating destination RMA memory. Aborting.\n");
		goto out;
	}

	for(i=0; i<N; i++)
		src[i] = 'r';

	memset( (void*) dst, 0, N);

	rofi_barrier();

	if(me)
		ret = rofi_iput(dst, src, N, 0, 0x0);
	       
	rofi_barrier();

	if(!me){
		for(i=0; i<N; i++)
			if(src[i] != dst[i])
				ret = 1;
		rofi_verify(ret);
	}

	rofi_release(src);
	rofi_release(dst);

 out:
	rofi_finit();
	return ret;
}
