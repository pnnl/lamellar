#include <pthread.h>
#include <assert.h>
#include <sys/mman.h>
#include <stdio.h>
#include <rdma/fi_rma.h>

#include <rofi_internal.h>
#include <rofi_debug.h>

#ifdef __APPLE__
#include <mach-o/getsect.h>
#else
/* Declare data_start and end as weak to avoid a linker error if the symbols
 * are not present.  During initialization we check if the symbols exist. */
#pragma weak __data_start
#pragma weak _end
extern int __data_start;
extern int _end;
#endif

rofi_mr_desc*     mr_tab = NULL;
pthread_rwlock_t  mr_lock;
void*             mr_next_addr;
uint64_t          mr_next_key = 0;
void*            ofi_data_base   = NULL;
unsigned long    ofi_data_length = 0;

extern struct fid_domain* domain;

int mr_init(void)
{
	pthread_rwlock_init(&mr_lock,NULL);
	mr_tab = NULL;

#ifdef __APPLE__
	ofi_data_base = (void*) get_etext();
	ofi_data_length = get_end() - get_etext();
#else
	if (&__data_start == (int*) 0 || &_end == (int*) 0)
		ERR_MSG("Unable to locate symmetric data segment (%p, %p)\n",
			(void*) &__data_start, (void*) &_end);
	
	ofi_data_base = (void*) &__data_start;
	ofi_data_length = (long) ((char*) &_end - (char*) &__data_start);
#endif
	mr_next_addr = (void*) (((unsigned long) ofi_data_base + ofi_data_length + GIGA) & ~(GIGA - 1));

	DEBUG_MSG("data base=%p length=%ld next address=%p", ofi_data_base, ofi_data_length, mr_next_addr);
	return 0;
}

rofi_mr_desc*  mr_add(size_t size, unsigned long mode)
{
	rofi_mr_desc *el, *tmp;
	void* addr = NULL;
	int err;
	unsigned long PageSize = rdesc.PageSize;

	el = malloc(sizeof(rofi_mr_desc));
	if(!(el)){
		ERR_MSG("Error allocating memory for memory region descriptor. Aborting!");
		goto err;
	}

	el->iov = (struct fi_rma_iov*) malloc(rdesc.nodes * sizeof(struct fi_rma_iov));
	if(!(el->iov)){
		ERR_MSG("Error allocating memory for remote memory region keys. Aborting!");
		goto err_el;
	}

	pthread_rwlock_wrlock(&mr_lock);
	HASH_FIND_PTR(mr_tab, &addr, tmp);
	if(tmp)
		goto err_lock;

	size = ((PageSize-1) & size) ? ((size + PageSize) & ~(PageSize-1)) : size;
	
	addr = mmap(mr_next_addr, size, PROT_READ | PROT_WRITE,
			     MAP_ANON | MAP_PRIVATE, -1, 0);

	if(addr == MAP_FAILED){
		perror("mmap");
		goto err_lock;
	}

	err = fi_mr_reg(domain, addr, size,
			FI_READ | FI_WRITE| FI_REMOTE_READ | FI_REMOTE_WRITE, 
			0, mr_next_key, 0x0,
			&(el->fid), &(el->ctx));	

	if(err != FI_SUCCESS){
		ERR_MSG("Error creating OFI memroy region (%d). Aborting.", err);
		goto err_mmap;
	}

	assert(err == 0);

	el->start   = addr;
 	el->size    = size;
	el->mode    = mode;
	el->mr_key  = fi_mr_key(el->fid);
	el->mr_desc = fi_mr_desc(el->fid);
	HASH_ADD_PTR(mr_tab, start, el);
	
	mr_next_addr += size;
	mr_next_key++;
	DEBUG_MSG("\t Added new memory region: ADDR=%p (%p) size=0x%lx mode=%u key=0x%lx desc=%p next=%p",
		  el->start, mr_next_addr, el->size, el->mode, el->mr_key, el->mr_desc, mr_next_addr);

 	pthread_rwlock_unlock(&mr_lock);
	return el;

 err_mmap:
 	munmap(addr,size);
 err_lock:
 	pthread_rwlock_unlock(&mr_lock);
 err_iov:
 	free(el->iov);
 err_el:
	free(el);
 err:
	return NULL;;
}

rofi_mr_desc* mr_get(const void* addr)
{
	rofi_mr_desc* el  = NULL;
	rofi_mr_desc* tmp = NULL;
	
	pthread_rwlock_rdlock(&mr_lock);
	HASH_ITER(hh, mr_tab, el, tmp){
		DEBUG_MSG("\t MR %p - %p size=%ld mode=0x%x",
			  el->start, el->start + el->size, el->size, el->mode);
		if(el->start <= addr && addr < (el->start + el->size))
			goto out;
	}
	el = NULL;	
	
 out:
	pthread_rwlock_unlock(&mr_lock);
	return el;
}

rofi_mr_desc* mr_get_from_remote(const void* remote_addr, unsigned long remote_id)
{
        rofi_mr_desc* el  = NULL;
        rofi_mr_desc* tmp = NULL;

        pthread_rwlock_rdlock(&mr_lock);
        HASH_ITER(hh, mr_tab, el, tmp){
                DEBUG_MSG("\t MR %p - %p size=%ld mode=0x%x",
                          el->start, el->start + el->size, el->size, el->mode);
                void* start = (void*)el->iov[remote_id].addr;
                void* end = start + el->size;
                if (start <= remote_addr && remote_addr <= end)
                        goto out;
        }
        el == NULL;

 out:
        pthread_rwlock_unlock(&mr_lock);
        return el;
}

int mr_rm(void* addr)
{
	rofi_mr_desc *el, *tmp;
	int ret = 0;

	pthread_rwlock_wrlock(&mr_lock);
	HASH_ITER(hh, mr_tab, el, tmp){
		if(el->start == addr){
			DEBUG_MSG("Removing memory region %p size 0x%lx", el->start, el->size);
			HASH_DEL(mr_tab, el);
			fi_close((struct fid*) el->fid);
			munmap(el->start, el->size);
			free(el->iov);
			free(el);
			goto out;
		}
	}
	ERR_MSG("Memory region %p not found!", addr);
	ret = -1;

 out:
	pthread_rwlock_unlock(&mr_lock);
	return ret;
}

int mr_free(void)
{
	rofi_mr_desc *el, *tmp;

	pthread_rwlock_wrlock(&mr_lock);
	HASH_ITER(hh, mr_tab, el, tmp){
		DEBUG_MSG("\t MR ADDR=%p size=%ld mode=0x%x",
			  el->start, el->size, el->mode);
		HASH_DEL(mr_tab, el);
		fi_close((struct fid*) el->fid);
		munmap(el->start, el->size);
		free(el->iov);
		free(el);
	}
	pthread_rwlock_unlock(&mr_lock);
	return 0;
}
