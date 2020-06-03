#include <unistd.h>
#include <stdio.h>
#include <string.h>
#include <time.h>

#include <rofi.h>
#include "utils.h"

#define N 1024UL*1024UL*1024UL
//#define VERBOSE

unsigned int  me, np;

static inline int verify_data(unsigned long* in, unsigned long* out, unsigned long size)
{
	unsigned long i;

	for(i=0; i<size; i++){
#ifdef VERBOSE
		printf("[%lu] %d: %lu\n", me, i, out[i]);
#endif
		if(in[i] != out[i]){
			printf("ERROR %lu: %lu != %lu\n", i, in[i], out[i]);
			return -1;
		}
	}

	return 0;
}

int main(void)
{
	unsigned int i;
	int ret = 0;
	unsigned long* src;
	unsigned long* target;
	struct timespec start, end;
      
	rofi_banner("PUT BW Test");
	rofi_init();
	
	np = rofi_get_size();
	if(np != 2){
		printf("Invalid number of processes (%u) (Required 2)! Aborting.\n", np);
		ret = -1;
		goto out;
	}

	me = rofi_get_id();

	ret = rofi_alloc(2*N*sizeof(unsigned long), 0x0, (void**) &src);
	if(ret){
		printf("Error allocating ROFI heap");
		goto out;
	}

	for(i=0; i<N; i++)
		src[i] = 10*i;
	
	target = &src[N];
	memset( (void*) target, 0, N*sizeof(unsigned long));

	rofi_barrier();

	if(me){
		clock_gettime(CLOCK_MONOTONIC, &start);
		if(rofi_iput(target, src, sizeof(unsigned long) * N, 0, 0x0)){
			printf("[%u] Error writing to remote node. Aborting...\n", me);
		}
		clock_gettime(CLOCK_MONOTONIC, &end);
	}

	rofi_barrier();
	
	if(me == 0)
		ret = verify_data(src, target, N);
	else{
                float rtime = ((float) tdiff(end,start))/BILLION;
                printf("  Sync test time: %f seconds\n", rtime);
                printf("  Throughput    : %f bytes/s\n", ((float) N*sizeof(unsigned long))/rtime);
	}
	   
	rofi_barrier();

	for(i=0; i<N; i++)
		src[i] = 20*i;
	
	memset( (void*) target, 0, N*sizeof(unsigned long));
	
	rofi_barrier();

	if(me){
		clock_gettime(CLOCK_MONOTONIC, &start);
		if(rofi_put(target, src, sizeof(unsigned long) * N, 0, 0x0)){
			printf("[%u] Error writing to remote node. Aborting...\n", me );
		}
		rofi_wait();
		clock_gettime(CLOCK_MONOTONIC, &end);
	}

	rofi_barrier();
	
	if(me == 0)
		ret = verify_data(src, target, N);
	else{
                float rtime = ((float) tdiff(end,start))/BILLION;
                printf("  Async test time: %f seconds\n", rtime);
                printf("  Throughput    : %f bytes/s\n", ((float) N*sizeof(unsigned long))/rtime);
	}
	
 out:
	rofi_verify(ret);
	rofi_finit();
	
	return 0;
}
