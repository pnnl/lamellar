#include <unistd.h>
#include <stdio.h>

#include <rofi.h>
#include "utils.h"

#define N 1
#define SIZE 128
//#define VERBOSE

int main(void)
{
	unsigned int me, np;
	unsigned int i;
	int ret = 0;
	char* addr1;
	char* addr2;
	char* addr3;
	
	rofi_init("verbs",NULL);
	
	np = rofi_get_size();
	me = rofi_get_id();

	if(!me)
		rofi_banner("Heap Alloc Test");

	printf("Process %d/%d allocating memory region of size %d\n", 
	       me, np, SIZE);
	
	ret = rofi_alloc(SIZE, 0x0, (void**) &addr1);
	if(ret){
		printf("Error allocating memory region! Aborting.");
	  goto out;
	}
	rofi_barrier();
	printf("Added memory region at %p size %d\n", addr1, SIZE);

	if(!me){
		printf("Address Mappig on all nodes:\n");
		for(i=0; i<np; i++)
			printf("\t Node %u: %d\n", i, rofi_get_remote_addr((void*) addr1, i));
	}

	ret = rofi_alloc(2 * SIZE, 0x0, (void**) &addr2);
	if(ret){
		printf("Error allocating memory region! Aborting.");
		goto out;
	}

	rofi_barrier();
	printf("Added memory region at %p size %d\n", addr2, SIZE);

	ret = rofi_release((void*) addr2);
	if(ret){
		printf("Error removing region %p!", addr2);
		goto out;
	}
	rofi_barrier();
	printf("Removed memory region at %p\n", addr2);

	ret = rofi_alloc(3 * SIZE, 0x0, (void**) &addr3);
	if(ret){
		printf("Error allocating memory region! Aborting.");
	  goto out;
	}

	rofi_barrier();
	printf("Added memory region at %p size %d\n", addr3, SIZE);

	ret = rofi_release((void*) addr1);
	if(ret){
		printf("Error removing region %p!", addr1);
		goto out;
	}
	rofi_barrier();
	printf("Removed memory region at %p\n", addr1);


	//Leave addr3 to be removed by ROFI

 out:	
	if(!me)
		rofi_verify(ret);
	rofi_finit();
	
	return 0;
}
