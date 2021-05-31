#include <unistd.h>
#include <stdio.h>
#include <string.h>
#include <time.h>
#include <math.h>
#include <stdint.h>

#include <rofi.h>
#include "utils.h"

#define N (1UL<<8)
//#define VERBOSE

static inline int verify_data(char* in, char* out, unsigned long size)
{
	unsigned long i;

	for(i=0; i<size; i++){
#ifdef VERBOSE
		printf("%lu: %c\n", i, out[i]);
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
	unsigned int i;
	int ret = 0, err = 0;
	uint32_t* sharray;
	uint32_t* results;
	unsigned int  me, np;
	uint32_t lres = 0;

	rofi_init("shm");
	np = rofi_get_size();
	if(np < 2){
		printf("Invalid number of processes (%u) (Required >= 2)! Aborting.\n", np);
		goto out;
	}
	
	me = rofi_get_id();

	if(!me)
		rofi_banner("ROFI Shared Array Test");

	ret = rofi_alloc(N * sizeof(uint32_t), 0x0, (void**) &sharray);
	if(ret){
		printf("Error allocating ROFI heap");
		goto out;
	}

	ret = rofi_alloc(np * sizeof(uint32_t), 0x0, (void**) &results);
	if(ret){
		printf("Error allocating ROFI heap");
		goto out;
	}
	
	for(i=0; i<N; i++)
		sharray[i] = me ? 0 : 1;
	
#ifdef VERBOSE
	for(i=0; i<N; i++)
		printf("Node %u: sarray[%u] = %u\n", me, i, sharray[i]);
#endif
	rofi_barrier();
	
	if(!me){
		printf("Node 0: distributing data...\n");
		for(i = 1; i < np; i++){
			if(rofi_iput(sharray, sharray, N * sizeof(uint32_t), i, 0x0))
				printf("Error writing to remote node %u!\n", i);
		}
	}

	rofi_barrier();

	printf("Node %u: computing partial sum...\n", me);
	for(i=0; i<N; i++)
		results[me] += sharray[i];
	printf("Node %u: Result of partial sum = %u\n", me, results[me]);

	rofi_barrier();

	if(me)
		if(rofi_iput(results + me, &(results[me]), sizeof(uint32_t), 0, 0x0))
			printf("Error transferring results to node 0!\n");

	rofi_barrier();
	
	if(!me){
		for(i=0; i<np; i++){
#ifdef VERBOSE
			printf("Node 0: results[%u] = %u\n", i, results[i]);
#endif
			lres += results[i];
		}
		printf("Node 0: overall results = %u (%lu)\n", lres, N * np);
	}

	rofi_barrier();

	rofi_release(results);
	rofi_release(sharray);

 out:
	rofi_finit();
	return 0;
}
