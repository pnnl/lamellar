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
#include <transport.h>
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

void*            ofi_heap_base   = NULL;
unsigned long    ofi_heap_length = 0;
unsigned long    ofi_heap_status = ROFI_HEAP_NOTALLOCATED;

ofi_ctx_t ofi_ctx;

fi_addr_t* remote_fi_addrs = NULL;
struct fi_rma_iov* remote_iov = NULL;;

extern struct fid_mr* mr;


#define GET_DEST(dest) ((fi_addr_t)(addr_table[(dest)]))

int  ft_init_fabric(void);
int  ft_finalize(void);
void ft_free_res(void);

void* rofi_get_remote_addr_internal(void* addr, unsigned int id)
{
	rofi_mr_desc* el = mr_get(addr);
	int ret = 0;

	if(!el)
		return NULL;
	
	DEBUG_MSG("\t Found MR [0x%lx - 0x%lx] Key: 0x%lx", el->start, el->start + el->size, el->mr_key);
	// printf("\t Found MR [0x%lx - 0x%lx] Key: 0x%lx iov: 0x%lx 0x%lx %p \n", el->start, el->start + el->size, el->mr_key, el->iov[id].addr,(void*) (addr - el->iov[id].addr +  (uintptr_t)el->start),(void*) (addr - (uintptr_t)el->start + el->iov[id].addr));
        //return (void*) (addr - el->iov[id].addr +  (uintptr_t)el->start);
        return (void*) (addr - (uintptr_t)el->start + el->iov[id].addr);
}

void* rofi_get_local_addr_from_remote_addr_internal(void* addr, unsigned int id)
{
	rofi_mr_desc* el = mr_get_from_remote(addr,id);
	int ret = 0;

	if(!el)
		return NULL;
	
	DEBUG_MSG("\t Found MR [0x%lx - 0x%lx] Key: 0x%lx", el->start, el->start + el->size, el->mr_key);
	// if (el->size <= 4096) {
	// 	printf("\t Found MR [0x%lx - 0x%lx] Key: 0x%lx remote_start 0x%lx offset 0x%lx len %lx local %lx\n", el->start, el->start + el->size, el->mr_key,el->iov[id].addr,addr - el->iov[id].addr, el->size,(addr - el->iov[id].addr +  (uintptr_t)el->start));
	// }
	return (void*) (addr - el->iov[id].addr +  (uintptr_t)el->start);	
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
	
	rdesc.tx_cntr = 0;
	rdesc.rx_cntr = 0;
}

void* rofi_alloc_internal(size_t size, unsigned long flags)
{
	rofi_mr_desc* el;
	int ret = 0;

	el = mr_add(size, flags);
	if(!el)
		return NULL;

	ret = ft_exchange_keys(el->iov, el->fid, el->start);
	if(ret)
		return NULL;

#ifdef _DEBUG
	for(int i=0; i< rdesc.nodes; i++)
		DEBUG_MSG("\t Node: %o Key: 0x%lx Addr: 0x%lx", i, el->iov[i].key, el->iov[i].addr);
#endif
	// for(int i=0; i< rdesc.nodes; i++)
	// 	printf("\t Node: %o Key: 0x%lx Addr: 0x%lx size: %lu\n", i, el->iov[i].key, el->iov[i].addr,size);
	// printf("\n");

	return el->start;
}

void* rofi_sub_alloc_internal(size_t size, unsigned long flags,uint64_t* pes, uint64_t num_pes)
{
	rofi_mr_desc* el;
	int ret = 0;

	el = mr_add(size, flags);
	if(!el)
		return NULL;
#ifdef _DEBUG
	for(int i=0; i< rdesc.nodes; i++)
		DEBUG_MSG("\t Node: %o Key: 0x%lx Addr: 0x%lx", i, el->iov[i].key, el->iov[i].addr);
#endif

	ret = ft_exchange_keys_sub(el->iov, el->fid, el->start,pes,num_pes);
	if(ret)
		return NULL;
		
#ifdef _DEBUG
	for(int i=0; i< rdesc.nodes; i++)
		DEBUG_MSG("\t Node: %o Key: 0x%lx Addr: 0x%lx", i, el->iov[i].key, el->iov[i].addr);
#endif
	return el->start;
}

int rofi_release_internal(void* addr)
{
	return mr_rm(addr);;
}

int rofi_sub_release_internal(void* addr,uint64_t* pes, uint64_t num_pes)
{
	return mr_rm(addr);;
}

unsigned int rofi_get_size_internal(void)
{
	return rdesc.nodes;
}

unsigned int rofi_get_id_internal(void)
{
	return rdesc.nid;
}

int rofi_put_internal(void* dst, void* src, size_t size, unsigned int id, unsigned long flags)
{
	rofi_mr_desc* el = mr_get(dst);
	struct fi_rma_iov rma_iov;
	int ret = 0;

#if 0
	if(rdesc.prov == shm && rdesc.tx_cntr >= FI_SHM_TX_SIZE)
		return EAGAIN;
#endif

	if(!fi_tx_size_left(ep))
		return EAGAIN;

	if(!el)
		goto err;
	
	DEBUG_MSG("\t Found MR [0x%p - 0x%p] Key: 0x%lx", el->start, el->start + el->size, el->mr_key);

	
	if ((fi->domain_attr->mr_mode == FI_MR_BASIC) ||
	    (fi->domain_attr->mr_mode & FI_MR_VIRT_ADDR)) {
		rma_iov.addr = (uint64_t) (dst - el->start + el->iov[id].addr);
	} else {
		rma_iov.addr = 0;
	}
	rma_iov.key = el->iov[id].key;
	DEBUG_MSG("\t Writing %lu bytes from %p to address 0x%lx at node %u with key 0x%lx (threshold %lu, in-flight msgs: %lu)",
		  size, src, rma_iov.addr, id, rma_iov.key, fi->tx_attr->inject_size, 
		  rdesc.tx_cntr);
	// printf("\t Writing %lu bytes from %p to address 0x%lx at node %u with key 0x%lx (threshold %lu, in-flight msgs: %lu\n)",
    //               size, src, rma_iov.addr, id, rma_iov.key, fi->tx_attr->inject_size,
    //               rdesc.tx_cntr);
	
	if (size < fi->tx_attr->inject_size) {
		DEBUG_MSG("\t Using RMA Inject");
		ainc(&rdesc.tx_cntr);
		ret = ft_post_rma_inject(FT_RMA_WRITE, ep, size, &rma_iov, src, id);
		adec(&rdesc.tx_cntr);
	} else {
		struct fi_context2* ctx = NULL;
		unsigned long txid = 0;
		ainc(&rdesc.tx_cntr);

		DEBUG_MSG("\t Using RMA POST");
		
		if(flags & ROFI_SYNC){
			ctx = (struct fi_context2*) malloc(sizeof(struct fi_context2));
			if(!ctx){
				ERR_MSG("Error allocating context for transmission.");
				goto err;
			}
		}else{
			txid = ctx_new();
			if(!txid){
				ERR_MSG("Error allocating context for new transaction.");
				goto err;
			}
			
			ctx = ctx_get(txid);
		}
		
		ret = ft_post_rma(FT_RMA_WRITE, ep, size, &rma_iov, src, id, el->mr_desc, 
				  &ctx);
		/*
		 * There is no immediate way in libfabrics to wait for a single transaction,
		 * unless different event queues are used. We need to wait for all previous
		 * transaction _and_ this one to be sure that all data has been transferred.
		 */
		if(flags & ROFI_SYNC){
			ctx_get_lock();
			ret = ft_get_tx_comp(tx_seq);
			ctx_cleanup();
			ctx_release_lock();
			free(ctx);
			adec(&rdesc.tx_cntr);
		}
	}
	
	return ret;
	
 err:
	return -1;
}

int rofi_get_internal(void* dst, void* src, size_t size, unsigned int id, unsigned long flags)
{
	rofi_mr_desc* el = mr_get(src);
	struct fi_rma_iov rma_iov;
	int ret = 0;
	struct fi_context2* ctx = NULL;
	unsigned long txid = 0;

#if 0
	if(rdesc.prov == shm && rdesc.rx_cntr >= FI_SHM_TX_SIZE)
		return EAGAIN;
#endif

	if(!fi_rx_size_left(ep))
		return EAGAIN;
	
	if(!el)
		goto err;
	
	DEBUG_MSG("\t Found MR [%p - %p] Key: 0x%lx", el->start, el->start + el->size, el->mr_key);

	if ((fi->domain_attr->mr_mode == FI_MR_BASIC) ||
	    (fi->domain_attr->mr_mode & FI_MR_VIRT_ADDR)) {
		rma_iov.addr = (uint64_t) (src - el->start + el->iov[id].addr);
	} else {
		rma_iov.addr = 0;
	}
	rma_iov.key = el->iov[id].key;
	DEBUG_MSG("\t Reading %lu bytes (into %p) from address 0x%lx at node %u with key 0x%lx (threshold %lu in-flight msgs: %lu)",
		  size, dst, rma_iov.addr, id, rma_iov.key, fi->tx_attr->inject_size, rdesc.rx_cntr);

	if(flags & ROFI_SYNC){
		ctx = (struct fi_context2*) malloc(sizeof(struct fi_context2));
		if(!ctx){
			ERR_MSG("Error allocating context for transmission.");
			goto err;
		}
	}else{
		txid = ctx_new();
		if(!txid){
			ERR_MSG("Error allocating context for new transaction.");
			goto err;
		}
		
		ctx = ctx_get(txid);
	}
	
	ainc(&rdesc.rx_cntr);
	ret = ft_post_rma(FT_RMA_READ, ep, size, &rma_iov, dst, id, el->mr_desc, 
			  ctx);
	
	assert(ret == 0);


	/*
	 * There is no immediate way in libfabrics to wait for a single transaction,
	 * unless different event queues are used. We need to wait for all previous
	 * transaction _and_ this one to be sure that all data has been transferred.
	 */
	if(flags & ROFI_SYNC){
		ctx_get_lock();
		ret = ft_get_tx_comp(tx_seq);
		ctx_cleanup();
		ctx_release_lock();
		free(ctx);
		adec(&rdesc.rx_cntr);
	}
	
       
	return ret;
	
 err:
	return -1;
}

int rofi_send_internal(unsigned long id, void* buf, size_t size, unsigned long flags)
{
	int ret = 0;

#if 0
	if(rdesc.prov == shm && rdesc.tx_cntr >= FI_SHM_TX_SIZE)
		return EAGAIN;
#endif

	if(!fi_tx_size_left(ep))
		return EAGAIN;


	memcpy((void*) (tx_buf + ft_tx_prefix_size()), buf, size );
	if (size < fi->tx_attr->inject_size)
		ret = ft_inject(ep, remote_fi_addrs[id], size);
	else
		ret = ft_tx(ep, remote_fi_addrs[id], size, &tx_ctx);
	if (ret)
		return ret;
	
	return 0;
}

int rofi_recv_internal(unsigned long id, void* buf, size_t size, unsigned long flags)
{
	int ret = 0;

#if 0
	if(rdesc.prov == shm && rdesc.rx_cntr >= FI_SHM_TX_SIZE)
		return EAGAIN;
#endif

	if(!fi_rx_size_left(ep))
		return EAGAIN;


	ret = ft_rx(ep, size);
	if(ret)
		return ret;
	memcpy(buf, (void*) rx_buf + ft_tx_prefix_size(), size);

	return 0;
}

int rofi_init_internal(char* prov)
{
	int ret = 0;
	
	opts = INIT_OPTS;
	opts.options |= FT_OPT_BW;
	ofi_heap_status = ROFI_HEAP_NOTALLOCATED;
	rdesc.PageSize = sysconf(_SC_PAGESIZE);
	rdesc.tx_cntr = 0;
	rdesc.rx_cntr = 0;

	ret = rt_init();
	if(ret){
		ERR_MSG("Error initializing ROFI RT. Aborting.");
		goto err;
	}

	rdesc.nodes = rt_get_size();
	rdesc.nid   = rt_get_rank();
	
	DEBUG_MSG("Initializing process %d/%d...", rdesc.nid, rdesc.nodes);

	hints = fi_allocinfo();
	if (!hints)
		return EXIT_FAILURE;

	hints->caps = FI_MSG | FI_RMA;
	hints->domain_attr->resource_mgmt = FI_RM_ENABLED;
	hints->mode = FI_CONTEXT;
	hints->domain_attr->threading = FI_THREAD_DOMAIN;
	hints->domain_attr->mr_mode = opts.mr_mode;

	if (!hints->fabric_attr) {
		hints->fabric_attr = malloc(sizeof *(hints->fabric_attr));
		if (!hints->fabric_attr) {
			perror("malloc");
			exit(EXIT_FAILURE);
		}
	}
	if(prov)
		hints->fabric_attr->prov_name = strdup(prov);
	hints->ep_attr->type = FI_EP_RDM;
	remote_fi_addrs = (fi_addr_t*) malloc(rdesc.nodes * sizeof(fi_addr_t));
	if(!remote_fi_addrs){
		ERR_MSG("Error allocating memory for remote addresses. Aborting!");
		return -ENOMEM;
	}
	
	for(int i=0; i<rdesc.nodes; i++)
		remote_fi_addrs[i] = FI_ADDR_UNSPEC;

	remote_iov = (struct fi_rma_iov*) malloc(rdesc.nodes * sizeof(struct fi_rma_iov));
	if(!remote_iov){
		ERR_MSG("Error allocating memory for remote memory region keys. Aborting!");
		return -ENOMEM;
	}
	
	ft_init_fabric();
	ret = ft_exchange_keys(remote_iov, mr, rx_buf + ft_rx_prefix_size());
	if(ret)
		return ret;
	
	mr_init();

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
	if (rdesc.nodes > 1){
		ft_sync();
	}
	ft_finalize();
	mr_free();
	
	ft_free_res();
	
	return 0;
}
