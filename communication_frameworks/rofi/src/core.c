#include <stdio.h>
#include <unistd.h>
#include <inttypes.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <sys/mman.h>
#include <rdma/fi_rma.h>

#include <rofi_debug.h>
#include <rofi_atomic.h>
#include <rofi_internal.h>
#include <rofi.h>

uint32_t     version = FI_VERSION(1,0);
rofi_desc_t  rdesc;

struct fi_info*    info      = NULL;
struct fid_fabric* ofi_ffid  = NULL;
struct fid_domain* ofi_dfid  = NULL;
struct fid_av*     ofi_avfid = NULL;
struct fid_ep*     ofi_epfid = NULL;
struct fid_cntr*   ofi_ctfid = NULL;
struct fid_cq*     ofi_cqfid = NULL;
struct fid_mr*     ofi_mrfd_heap = NULL;
struct fid_mr*     ofi_mrfd_data = NULL;
struct fid_stx*    ofi_stxfid = NULL;

void*            ofi_data_base   = NULL;
unsigned long    ofi_data_length = 0;
void*            ofi_heap_base   = NULL;
unsigned long    ofi_heap_length = 0;
unsigned long    ofi_heap_status = ROFI_HEAP_NOTALLOCATED;

ofi_ctx_t ofi_ctx;

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

#define GET_DEST(dest) ((fi_addr_t)(addr_table[(dest)]))

int ofi_get_mr(const void* addr, unsigned int pe, uint8_t** mr_addr, uint64_t* key)
{
	if ((void*) addr >= ofi_data_base &&
	    (uint8_t*) addr < (uint8_t*) ofi_data_base + ofi_data_length) {
		
		*key = 0;
		*mr_addr = (uint8_t*) ((uint8_t *) addr - (uint8_t *) ofi_data_base);
		
	} else if ((void*) addr >= ofi_heap_base &&
		   (uint8_t*) addr < (uint8_t*) ofi_heap_base + ofi_heap_length) {
		
		*key = 1;
		*mr_addr = (uint8_t*) ((uint8_t *) addr - (uint8_t *) ofi_heap_base);
	} else {
		*key = 0;
		*mr_addr = NULL;
		ERR_MSG("address (%p) outside of symmetric areas\n", addr);
		return -1; 
	}
	
	return 0;
}



int rofi_put_internal(void* dst, const void* src, size_t size, unsigned int id, unsigned long flags)
{
	int ret = 0;
	uint64_t key;
	uint8_t *addr;
	struct fi_cq_entry cqe;

	ofi_get_mr(dst, id, &addr, &key);

	ret = fi_write(ofi_ctx.ep, src, size, NULL, id, (uint64_t) addr, key, NULL);
	while (ret == -FI_EAGAIN){
		ret = fi_write(ofi_ctx.ep, src, size, NULL, id, (uint64_t) addr, key, NULL);
		sched_yield();
	}		
	
	if(ret){
		ERR_MSG("%p %p %lu %u %lu\n",dst,src,size,id,flags);
		perror("fi_inject_write");
		goto out;
	}
	
	if(flags & ROFI_ASYNC){
		return 0;
	}
	
	while(1){
		ret = fi_cq_read(ofi_ctx.cq, &cqe, 1);
		if(ret > 0){
			DEBUG_MSG("\tfi_cq_read Returned %d completion evetns.", ret);
			return 0;
		}

		if(ret != -FI_EAGAIN){
			struct fi_cq_err_entry cqe_err;
			fi_cq_readerr(ofi_ctx.cq, &cqe_err, 0);
			ERR_MSG("\t%s %s", fi_strerror(cqe_err.err), 
				fi_cq_strerror(ofi_ctx.cq, cqe_err.prov_errno, cqe_err.err_data,
					       NULL, 0));
			return ret;
		}		
	}

 out:
	return ret;

}

int rofi_wait_internal(void)
{
	int ret = 0;
	uint64_t key;
	struct fi_cq_entry cqe;

	while(1){
		ret = fi_cq_read(ofi_ctx.cq, &cqe, 1);
		if(ret > 0){
			DEBUG_MSG("\tfi_cq_read Returned %d completion evetns.", ret);
			return 0;
		}
		
		if(ret != -FI_EAGAIN){
			struct fi_cq_err_entry cqe_err;
			fi_cq_readerr(ofi_ctx.cq, &cqe_err, 0);
			ERR_MSG("\t%s %s", fi_strerror(cqe_err.err), 
				fi_cq_strerror(ofi_ctx.cq, cqe_err.prov_errno, cqe_err.err_data,
					       NULL, 0));
			return ret;
		}		
	}
	

}

int rofi_get_internal(void* dst, const void* src, size_t size, unsigned int id, unsigned long flags)
{
	int ret = 0;
	uint64_t key;
	uint8_t * addr;
	struct fi_cq_entry cqe;

	ofi_get_mr(src, id, &addr, &key);

	ret = fi_read(ofi_ctx.ep, dst, size, NULL, id, (uint64_t) addr, key, NULL);
	while (ret == -FI_EAGAIN){
		ret = fi_read(ofi_ctx.ep, dst, size, NULL, id, (uint64_t) addr, key, NULL);
		sched_yield();
	}

	if(ret){
		perror("fi_read");
		goto out;
	}

	if(flags & ROFI_ASYNC)
		return 0;

	while(1){
		ret = fi_cq_read(ofi_ctx.cq, &cqe, 1);
		if(ret > 0){
			DEBUG_MSG("\tfi_cq_read Returned %d completion evetns.", ret);
			return 0;
		}

		if(ret != -FI_EAGAIN){
			struct fi_cq_err_entry cqe_err;
			fi_cq_readerr(ofi_ctx.cq, &cqe_err, 0);
			ERR_MSG("\t%s %s", fi_strerror(cqe_err.err), 
				fi_cq_strerror(ofi_ctx.cq, cqe_err.prov_errno, cqe_err.err_data,
					       NULL, 0));
			return ret;
		}		
	}

 out:
	return ret;
}

int rofi_release_internal(void)
{

	if(!bcas(&ofi_heap_status, ROFI_HEAP_REGISTERED, ROFI_HEAP_ALLOCATED)){
		DEBUG_MSG("Symmeetric heap not allocated or being released by another thread.");
		goto err;
	}
		
	if(munmap(ofi_heap_base,ofi_heap_length)){
		ERR_MSG("Error unmapping symmetric heap!");
		perror("munmap");
		goto err;
	}

	fi_close(&(ofi_mrfd_heap->fid));
	ofi_heap_base   = NULL;
	ofi_heap_length = 0;

	cas(&ofi_heap_status, ROFI_HEAP_ALLOCATED, ROFI_HEAP_NOTALLOCATED);

	return 0;
 err:
	return -1;
}

void* rofi_alloc_internal(size_t size, unsigned long flags)
{
	void* ret = NULL;
	void *requested_base = NULL;
	int  cret = 0;

	/* If this fails, the heap has been already allocated or someone else is tryihng to
	 * allocated the symmetric heap and has made further progress than us.
	 */
	if(!bcas(&ofi_heap_status, ROFI_HEAP_NOTALLOCATED, ROFI_HEAP_ALLOCATED)){
		DEBUG_MSG("Symmetric heap has been already allocated or is being allocated by other threads.");
		goto err;
	}
	
	requested_base = (void*) (((unsigned long) ofi_data_base +
				   ofi_data_length + GIGA) & ~(GIGA - 1));

	ret = mmap(requested_base, size, PROT_READ | PROT_WRITE,
			     MAP_ANON | MAP_PRIVATE, -1, 0);
	if (ret == MAP_FAILED) {
		ERR_MSG("Error allocating symmetric heap (%d)", errno);
		perror("mmap");
		ofi_heap_status = ROFI_HEAP_INVALID;
		goto err;
	}

	cret = fi_mr_reg(ofi_dfid, ret, size,
			 FI_REMOTE_READ | FI_REMOTE_WRITE, 0, 1ULL, 0x0,
			 &ofi_mrfd_heap, NULL);	
	if(cret){
		ERR_MSG("Error while registeing symmetric heap (%d)!", cret);
		goto err_mmap;
	}

	cret = fi_mr_bind(ofi_mrfd_heap, &ofi_ctfid->fid, FI_REMOTE_WRITE);
	if (cret) {
		ERR_MSG("Error binding heap MR (%d)", cret);
		perror("fi_mr_bind");
		goto err_mmap;
	}

	ofi_heap_base   = ret;
	ofi_heap_length = size;

	cas(&ofi_heap_status, ROFI_HEAP_ALLOCATED, ROFI_HEAP_REGISTERED);

	return ret;

 err_mmap:
	munmap(ret, size);
 err:	
	return NULL;
}

unsigned int rofi_get_size_internal(void)
{
	return rdesc.nodes;
}

unsigned int rofi_get_id_internal(void)
{
	return rdesc.nid;
}

static inline int ofi_init(void)
{
	int ret = 0;
	struct fi_info hints = {0};
	struct fi_domain_attr domain_attr = {0};
	struct fi_ep_attr ep_attr = {0};
	struct fi_fabric_attr fabric_attr = {0};
	struct fi_tx_attr tx_attr = {0};
	struct fi_av_attr av_attr = {0};
	struct fi_cntr_attr cntr_attr = {0};
	struct fi_cq_attr  cq_attr = {0};
	char   epname[128];
	size_t epnamelen = sizeof(epname);	

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
	
	hints.caps = FI_RMA | FI_ATOMIC | FI_RMA_EVENT | FI_FENCE;
	hints.addr_format = FI_FORMAT_UNSPEC;

	domain_attr.data_progress = FI_PROGRESS_AUTO;
	domain_attr.resource_mgmt = FI_RM_ENABLED;
	/*
	  FI_MR_RMA_EVENT doesn't seem to be supported on Mac
	 */
	domain_attr.mr_mode = FI_MR_SCALABLE;
	domain_attr.mr_key_size = 1;
	domain_attr.threading = FI_THREAD_DOMAIN;

	ep_attr.type = FI_EP_RDM;
	ep_attr.tx_ctx_cnt = 0;
	
	tx_attr.op_flags = FI_DELIVERY_COMPLETE;
	tx_attr.inject_size = sizeof(long double);

	av_attr.type = FI_AV_TABLE;

	cntr_attr.events = FI_CNTR_EVENTS_COMP;
	cntr_attr.wait_obj = FI_WAIT_UNSPEC;
	
	hints.domain_attr = &domain_attr;
	hints.fabric_attr = &fabric_attr;
	hints.tx_attr =&tx_attr;
	hints.rx_attr = NULL;
	hints.ep_attr = &ep_attr;
	
	DEBUG_MSG("\tInitializing OFI transport layer...");

	ret = fi_getinfo(version,  NULL, NULL, 0 , &hints, &info);
	if(ret){
		ERR_MSG("Error getting info from OFI transport layer (%d)", ret);
		goto err_data;
	}

#ifdef _DEBUG
	for(struct fi_info* p = info; p != NULL; p = p->next)
		DEBUG_MSG("\tFound fabric %s provider %s",
			  p->fabric_attr->name,
			  p->fabric_attr->prov_name);
#endif

	if(info->ep_attr->max_msg_size <=0){
		ERR_MSG("Provider did not set max_msg_size. Aborting");
		goto err;
	}
	
	info->domain_attr->mr_key_size = 0;

	DEBUG_MSG("\tSelected 1st Provider: %s Version: (%u.%u) Fabric: %s Domain: %s max_inject: %zu, max_msg: %zu, stx: %s",
		  info->fabric_attr->prov_name,
		  FI_MAJOR(info->fabric_attr->prov_version),
		  FI_MINOR(info->fabric_attr->prov_version),
		  info->fabric_attr->name,
		  info->domain_attr->name,
		  info->tx_attr->inject_size,
		  info->ep_attr->max_msg_size,
		  info->domain_attr->max_ep_stx_ctx == 0 ? "no" : "yes");


	ret = fi_fabric(info->fabric_attr, &ofi_ffid, NULL);
	if(ret){
		ERR_MSG("Error initializing OFI fabric (%d)", ret);
		goto err_info;
	}
	DEBUG_MSG("OFI Fabric initialized.");
	DEBUG_MSG("OFI version: built %"PRIu32".%"PRIu32", cur. %"PRIu32".%"PRIu32"; "
              "provider version: %"PRIu32".%"PRIu32"\n",
              FI_MAJOR_VERSION, FI_MINOR_VERSION,
              FI_MAJOR(fi_version()), FI_MINOR(fi_version()),
              FI_MAJOR(info->fabric_attr->prov_version),
              FI_MINOR(info->fabric_attr->prov_version));

	
	ret = fi_domain(ofi_ffid, info, &ofi_dfid, NULL);
	if(ret){
		ERR_MSG("Error initializing OFI Domain (%d)", ret);
		goto err_fabric;
	}
	DEBUG_MSG("OFI Domain initialized.");

	ret = fi_av_open(ofi_dfid, &av_attr, &ofi_avfid, NULL);
	if(ret){
		ERR_MSG("Error initializing OFU AV (%d)", ret);
		goto err_domain;
	}
	DEBUG_MSG("OFI AV initialized.");

	info->ep_attr->tx_ctx_cnt = 0;
	info->caps = FI_RMA | FI_ATOMIC | FI_REMOTE_READ | FI_REMOTE_WRITE;
	info->tx_attr->op_flags = 0;
	info->mode = 0;
	info->tx_attr->mode = 0;
	info->rx_attr->mode = 0;

	ret = fi_endpoint(ofi_dfid, info, &ofi_epfid, NULL);
	if(ret){
		ERR_MSG("Error initializing OFI end pont (%d)", ret);
		goto err_av;
	}
	DEBUG_MSG("OFI EP initialized.");
	
	ret = fi_ep_bind(ofi_epfid, &(ofi_avfid->fid), 0x0);
	if(ret){
		ERR_MSG("Error binding EP to AV (%d)", ret);
		goto err_ep;
	}
	DEBUG_MSG("OFI EP bound to AV.");

	ret = fi_cntr_open(ofi_dfid, &cntr_attr, &ofi_ctfid, NULL);
	if (ret) {
		ERR_MSG("Error initializing OFI fabric CNTRs (%d)", ret);
		goto err_ep_av;
	}
	
	ret = fi_mr_reg(ofi_dfid, ofi_data_base,
			ofi_data_length,
			FI_REMOTE_READ | FI_REMOTE_WRITE, 0, 0ULL, 0x0,
			&ofi_mrfd_data, NULL);
	if (ret) {
		ERR_MSG("Error registering data MR (%d)", ret);
		goto err_ep_av;
	}
	
	ret = fi_mr_bind(ofi_mrfd_data, &ofi_ctfid->fid, FI_REMOTE_WRITE);
	if (ret) {
		ERR_MSG("Error binding data MR (%d)", ret);
		goto err_ep_av;
	}

	ret = fi_cq_open(ofi_dfid, &cq_attr, &ofi_cqfid, NULL);
	if(ret){
		ERR_MSG("Error initializing OFI command queue (%d)", ret);
		goto err_ep_av;
	}
	DEBUG_MSG("OFI Command Queuee initialized.");

	ret = fi_ep_bind(ofi_epfid, &ofi_cqfid->fid, FI_RECV);
	if(ret){
		ERR_MSG("Error binding OFI command queue (%d)", ret);
		goto err_ep_av;
	}
	DEBUG_MSG("OFI Command Queuee bound.");

	ret = fi_enable(ofi_epfid);
	if(ret){
		ERR_MSG("Failing to enable end point (%d)", ret);
		goto err_ep_av;
	}
	DEBUG_MSG("OFI EP enabled.");

	ret = fi_getname((fid_t)ofi_epfid, epname, &epnamelen);
	if(ret || (epnamelen > sizeof(epname))){
		ERR_MSG("Error getting EP name (%d, %zu).", ret, epnamelen);
		goto err_ep_av;
	}
	rdesc.addrlen = epnamelen;
	
	ret = rt_put("fi_epname", epname, epnamelen);
	if(ret){
		ERR_MSG("Error adding epname to KVS.");
		goto err_ep_av;
	}
	
	DEBUG_MSG("EP name: %s (%zu)", epname, epnamelen);
	return ret;

#if 0	
 err_bind:
	fi_shutdown(fi->epfid,0);
 err_ep:
	fi_shutdown(fi->epfid,0);
 err_cq:
	fi_close(&(fi->cqfid->fid)); 
 err_eq:
	fi_close(&(fi->eqfid->fid));
#endif
 err_ep_av:
 err_ep:
	fi_close(&(ofi_epfid->fid)); 	
 err_av:
	fi_close(&(ofi_avfid->fid));
 err_domain:
	fi_close(&(ofi_dfid->fid)); 
 err_fabric:
	fi_close(&(ofi_ffid->fid)); 
 err_info:
	fi_freeinfo(info);
 err_data:
	free(ofi_data_base);
 err:
	return ret;

}

static inline int ofi_finit(void)
{
	  int ret = 0;

	  fi_close(&(ofi_epfid->fid)); 	
	  fi_close(&(ofi_avfid->fid)); 
	  fi_close(&(ofi_dfid->fid)); 
	  fi_close(&(ofi_ffid->fid));
	  fi_freeinfo(info);

	  return ret; 
}

int ofi_startup(ofi_ctx_t* ctx)
{
	int ret = 0;
	struct fi_cntr_attr cntr_put_attr = {0};
	struct fi_cntr_attr cntr_get_attr = {0};
	struct fi_cq_attr cq_attr = {0};
	char* alladdrs = NULL;
	
	ret = fi_stx_context(ofi_dfid, NULL, &ofi_stxfid, NULL);
	if(ret){
		ERR_MSG("Error creating OFI STX (%d)", ret);
		goto err;
	}

	cntr_put_attr.events   = FI_CNTR_EVENTS_COMP;
	cntr_get_attr.events   = FI_CNTR_EVENTS_COMP;
	cntr_put_attr.wait_obj = FI_WAIT_UNSPEC;
	cntr_get_attr.wait_obj = FI_WAIT_UNSPEC;

	cq_attr.format = FI_CQ_FORMAT_CONTEXT;

	info->ep_attr->tx_ctx_cnt = FI_SHARED_CONTEXT;
	info->caps = FI_RMA | FI_WRITE | FI_READ | FI_ATOMIC;
	info->tx_attr->op_flags = FI_DELIVERY_COMPLETE;
	info->mode = 0;
	info->tx_attr->mode = 0;
	info->rx_attr->mode = 0;

	ret = fi_cntr_open(ofi_dfid, &cntr_put_attr, &ctx->put_cntr, NULL);
	if(ret){
		ERR_MSG("Error opening CTX PUT counter.");
		goto err;
	}

	ret = fi_cntr_open(ofi_dfid, &cntr_get_attr, &ctx->get_cntr, NULL);
	if(ret){
		ERR_MSG("Error opening CTX GET counter.");
		goto err;
	}

	ret = fi_cq_open(ofi_dfid, &cq_attr, &ctx->cq, NULL);
	if(ret){
		ERR_MSG("Error opening CTX CQ.");
		goto err;
	}

	ret = fi_endpoint(ofi_dfid, info, &ctx->ep, NULL);
	if(ret){
		ERR_MSG("Error opening CTX EP.");
		goto err;
	}

	ret = fi_ep_bind(ctx->ep, &ofi_stxfid->fid, 0);
	if(ret){
		ERR_MSG("Error binding CTX EP/STX.");
		goto err;
	}

	ret = fi_ep_bind(ctx->ep, &ctx->put_cntr->fid, FI_WRITE);
	if(ret){
		ERR_MSG("Error binding CTX EP/PUT CNTR.");
		goto err;
	}

	ret = fi_ep_bind(ctx->ep, &ctx->get_cntr->fid, FI_READ);
	if(ret){
		ERR_MSG("Error binding CTX EP/GET CNTR.");
		goto err;
	}

	//	ret = fi_ep_bind(ctx->ep, &ctx->cq->fid, FI_SELECTIVE_COMPLETION | FI_TRANSMIT | FI_RECV);
	ret = fi_ep_bind(ctx->ep, &ctx->cq->fid, FI_TRANSMIT | FI_RECV);
	if(ret){
		ERR_MSG("Error binding CTX EP/CQ.");
		goto err;
	}

	ret = fi_ep_bind(ctx->ep, &ofi_avfid->fid, 0);
	if(ret){
		ERR_MSG("Error binding CTX EP/AV.");
		goto err;
	}

	ret = fi_enable(ctx->ep);
	if(ret){
		ERR_MSG("Error enabling CTX EP.");
		goto err;
	}

	alladdrs = (char*) malloc(rdesc.nodes * rdesc.addrlen);
	if(!alladdrs){
		ERR_MSG("Error allocating memory for addresses.");
		goto err;
	}

	for(int i = 0; i < rdesc.nodes; i++){
		char* ptr = alladdrs + i * rdesc.addrlen;
		ret = rt_get(i, "fi_epname", ptr, rdesc.addrlen);
		if(ret){
			ERR_MSG("Error getting EP address name from %i (%d).", i, ret);
			goto err;
		}
	}

	ret = fi_av_insert(ofi_avfid, alladdrs, rdesc.nodes, NULL, 0, NULL);
	if(ret != rdesc.nodes){
		ERR_MSG("Error inserting %d addresses into AV table (%d).", rdesc.nodes, ret);
		goto err;
	}

	free(alladdrs);
	return 0;
 err:
	return -1;
}

int rofi_init_internal(void)
{
	int ret = 0;
	
	ofi_heap_status = ROFI_HEAP_NOTALLOCATED;

	ret = rt_init();
	if(ret){
		ERR_MSG("Error initializing ROFI RT. Aborting.");
		goto err;
	}

	rdesc.nodes = rt_get_size();
	rdesc.nid   = rt_get_rank();
	
	DEBUG_MSG("Initializing process %d/%d...", rdesc.nid, rdesc.nodes);

	ret = ofi_init();
	if(ret){
		ERR_MSG("Error initializing OFI transport layer. Aborting.");
		goto err;
	}

	ret = rt_exchange();
	if(ret){
		ERR_MSG("Error exchanging runtime information. Aborting.");
		goto err;
	}

	ret = ofi_startup(&ofi_ctx);
	if(ret){
		ERR_MSG("Error starting OFI transport layer. Aborting.");
		goto err;
	}

	rdesc.status = ROFI_STATUS_ACTIVE;
	
	return 0;
	
 err:
	rdesc.status = ROFI_STATUS_ERR;
	return -1;
}


/*
 * Unmap symmetric heap after OFU transport is down to ensure nobody accidentally writes
 * to an unmapped heap.
 */
int rofi_finit_internal(void)
{
	rdesc.status = ROFI_STATUS_TERM;

	ofi_finit();
	rt_finit();

	if(ofi_heap_status == ROFI_HEAP_REGISTERED)
		rofi_release_internal();

	return 0;
}
