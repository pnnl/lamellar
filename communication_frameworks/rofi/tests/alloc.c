#include <unistd.h>
#include <stdio.h>

#include <rofi.h>
#include "utils.h"

#define N 128*1024*1024UL
//#define VERBOSE

int main(void)
{
	unsigned int me, np;
	unsigned long long i;
	int ret = 0;
	unsigned long* src;
	unsigned long* target;
	
	rofi_banner("Heap Alloc Test");
	rofi_init();
	
	np = rofi_get_size();
	me = rofi_get_id();
	if(np != 2){
		printf("Required 2 processes, provided %d. Aborting.", np);
		ret = 1;
		goto out;
	}

	printf("Process %d/%d allocating memory region of size %lu\n", 
	       me, np, 2*N*sizeof(unsigned long));

	ret = rofi_alloc(2*N*sizeof(unsigned long), 0x0, (void**) &src);
	if(ret){
		printf("Error allocating memory region! Aborting.");
		goto out;
	}

	for(i=0; i<N; i++)
		src[i] = 10*i;

	target = &(src[N]);

	rofi_barrier();
	
	if(me){
		if(rofi_iget(target, src, sizeof(unsigned long) * N, 0, 0x0)){
			printf("Error while getting data from node 0. Aborting!");
			ret = 1;
			goto out;
		}

		for(i=0; i<N; i++){
			if(src[i] != target[i]){
				printf("%llu: %lu != %lu\n", i, src[i], target[i]);
				ret = 1;
				goto out;
			}
		}
	}			
	
 out:	
	rofi_barrier();
	//	rofi_release();
	rofi_verify(ret);
	rofi_finit();
	
	return 0;
}
