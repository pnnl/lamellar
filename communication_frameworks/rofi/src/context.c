#include <pthread.h>
#include <assert.h>
#include <sys/mman.h>
#include <stdio.h>
#include <rdma/fi_rma.h>

#include <rofi_internal.h>
#include <rofi_debug.h>


rofi_ctx_desc*           ctx_tab = NULL;
pthread_rwlock_t         ctx_lock;
volatile unsigned long   current_txID;

int ctx_init(void)
{
	pthread_rwlock_init(&ctx_lock,NULL);
	ctx_tab = NULL;

	current_txID = 0;

	return 0;
}

unsigned long ctx_new(void)
{
	rofi_ctx_desc *el, *tmp;
	int err;

	el = malloc(sizeof(rofi_ctx_desc));
	if(!(el)){
		ERR_MSG("Error allocating memory for context descriptor. Aborting!");
		goto err;
	}

	el->ctx = (struct fi_context2*) malloc(sizeof(struct fi_context2));
	if(!(el->ctx)){
		ERR_MSG("Error allocating memory for FI context. Aborting!");
		goto err_el;
	}

	pthread_rwlock_wrlock(&ctx_lock);
	el->txid = ++current_txID;

	HASH_FIND_INT(ctx_tab, &(el->txid), tmp);
	if(tmp)
		goto err_lock;

	HASH_ADD_INT(ctx_tab, txid, el);
	
	DEBUG_MSG("\t Added new transaction context: TxID = 0x%lx context addr = %p",
		  el->txid, el->ctx);

 	pthread_rwlock_unlock(&ctx_lock);
	return el->txid;

 err_lock:
 	pthread_rwlock_unlock(&ctx_lock);
 err_iov:
 	free(el->ctx);
 err_el:
	free(el);
 err:
	return 0;
}

struct fi_context2* ctx_get(const unsigned long id)
{
	rofi_ctx_desc* el  = NULL;
	
	pthread_rwlock_rdlock(&ctx_lock);
	HASH_FIND_INT(ctx_tab, &id, el);	
	pthread_rwlock_unlock(&ctx_lock);

	if(el)
		return el->ctx;
	else
		return NULL;
}

int ctx_rm(unsigned long id)
{
	rofi_ctx_desc *el, *tmp;
	int ret = 0;

	pthread_rwlock_wrlock(&ctx_lock);
	HASH_ITER(hh, ctx_tab, el, tmp){
		if(el->txid == id){
			DEBUG_MSG("Removing TX contextn 0x%lx", el->txid);
			HASH_DEL(ctx_tab, el);
			free(el->ctx);
			free(el);
			goto out;
		}
	}
	ERR_MSG("Context ID 0x%lx not found!", id);
	ret = -1;

 out:
	pthread_rwlock_unlock(&ctx_lock);
	return ret;
}

/*
 * This function must be called with the ctx_lock already acquired. The caller
 * also needs to release the lock once this function returns.
 */
int ctx_cleanup(void)
{
	rofi_ctx_desc *el, *tmp;

	HASH_ITER(hh, ctx_tab, el, tmp){
		DEBUG_MSG("\t Removing TX context 0x%lx", el->txid);
		HASH_DEL(ctx_tab, el);
		free(el->ctx);
		free(el);
	}

	return 0;
}

int ctx_free(void)
{
	rofi_ctx_desc *el, *tmp;

	pthread_rwlock_wrlock(&ctx_lock);
	HASH_ITER(hh, ctx_tab, el, tmp){
		DEBUG_MSG("\t Removing TX context 0x%lx", el->txid);
		HASH_DEL(ctx_tab, el);
		free(el->ctx);
		free(el);
	}
	pthread_rwlock_unlock(&ctx_lock);

	return 0;
}

int ctx_get_lock(void)
{
	pthread_rwlock_wrlock(&ctx_lock);
	return 0;
}

int ctx_release_lock(void)
{
	pthread_rwlock_unlock(&ctx_lock);
	return 0;
}


