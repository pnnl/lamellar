#include <unistd.h>
#include <stdio.h>

#include <rofi.h>
#include "utils.h"

#define N 16

unsigned long source[N];
unsigned long target[N];

int main(void)
{
	unsigned int me, np, i;
	int ret = 0;

	rofi_banner("Ping Pong Test");
	rofi_init();
	
	np = rofi_get_size();
	if(np != 2){
		printf("Invalid number of processes (%u) (Required 2)! Aborting.\n", np);
		ret = -1;
		goto out;
	}

	me = rofi_get_id();

	for(i=0; i<N; i++){
		source[i] = i;
		target[i] = 0;
	}
	
	rofi_barrier();

	if(me == 0)
		if(rofi_put(target, source, sizeof(unsigned long) * N, 1, 0x0)){
			printf("[%u] Error writing to remote node. Aborting...\n", me );
			ret = 1;
			goto out;
		}

	rofi_barrier();

	if(me == 1){
		sleep(1);
		for(i=0; i<N; i++){
			//			printf("[%lu] %d: %lu\n", me, i, target[i]);
			if(source[i] != target[i]){
				printf("ERROR %d: %lu != %lu\n", i, source[i], target[i]);
				ret = 1;
			}
		}
		if(ret)
			goto out;
	}
	   
	for(i=0; i<N; i++)
		source[i] *= 10;

	rofi_barrier();

	if(me == 0)
		if(rofi_get(target, source, sizeof(unsigned long) * N, 1, 0x0)){
			printf("[%u] Errro reading from remote node. Aborting...\n", me);
			//			ret = 1;
			//			goto out;
		}

	rofi_barrier();

	if(me == 0){
		sleep(1);
		for(i=0; i<N; i++){
			//			printf("[%lu] %d: %lu\n", me, i, target[i]);
			if(source[i] != target[i]){
				printf("ERROR %d: %lu != %lu\n", i, source[i], target[i]);
				//				ret = 1;
				//				goto out;
			}
		}
		if(ret)
			goto out;
	}

 out:
	rofi_verify(ret);
	rofi_finit();
	
	return 0;
}
