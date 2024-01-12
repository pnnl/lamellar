#include <assert.h>
#include <inttypes.h>
#include <math.h>
#include <rdma/fi_rma.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#include <rofi.h>
#include <rofi_atomic.h>
#include <rofi_debug.h>
#include <rofi_internal.h>
#include <transport.h>



rofi_transport_t rofi;

void *rofi_get_remote_addr_internal(void *addr, unsigned int id) {
    rofi_mr_desc *el = mr_get(addr);
    int ret = 0;

    if (!el)
        return NULL;

    DEBUG_MSG("\t Found MR [0x%lx - 0x%lx] Key: 0x%lx %p", el->start, el->start + el->size, el->mr_key, (void *)(addr - (uintptr_t)el->start + el->iov[id].addr));
    return (void *)(addr - (uintptr_t)el->start + el->iov[id].addr);
}

void *rofi_get_local_addr_from_remote_addr_internal(void *addr, unsigned int id) {
    rofi_mr_desc *el = mr_get_from_remote(addr, id);
    int ret = 0;

    if (!el)
        return NULL;

    DEBUG_MSG("\t Found MR [0x%lx - 0x%lx] Key: 0x%lx", el->start, el->start + el->size, el->mr_key);
    return (void *)(addr - el->iov[id].addr + (uintptr_t)el->start);
}

int rofi_wait_internal(void) {
    rofi_transport_put_wait_all(&rofi);
    rofi_transport_get_wait_all(&rofi);
    return 0;
}

void *rofi_alloc_internal(size_t size, unsigned long flags) {

    rofi_mr_desc *mr = mr_add(&rofi,size, flags);
    if (!mr) {
        return NULL;
    }

    if (rofi_transport_exchange_mr_info(&rofi, mr)) {
        return NULL;
    }

    return mr->start;
}

void *rofi_sub_alloc_internal(size_t size, unsigned long flags, uint64_t *pes, uint64_t num_pes) {
}

int rofi_release_internal(void *addr) {
    return mr_rm(addr);
}

int rofi_sub_release_internal(void *addr, uint64_t *pes, uint64_t num_pes) {
    return mr_rm(addr);
}

unsigned int rofi_get_size_internal(void) {
    return rofi.desc.nodes;
}

unsigned int rofi_get_id_internal(void) {
    return rofi.desc.nid;
}

// NOTE this is needed to ensure progress for something like n-way dissemination barriers
// as recipients of RDMA Put still need to proress their completion queues for others to continue
int rofi_flush_internal(void) {
    // return ft_get_rx_comp(0);
    return 0;
}

int rofi_put_internal(void *dst, void *src, size_t size, unsigned int id, unsigned long flags) {
    rofi_mr_desc *el = mr_get(dst);
    struct fi_rma_iov rma_iov;
    int ret = 0;

    if (!el) {
        DEBUG_MSG("\t No mr found for address %p on node %u", dst, id);
        return -1;
    }
    DEBUG_MSG("\t Found MR [0x%p - 0x%p] Key: 0x%lx", el->start, el->start + el->size, el->mr_key);

    rma_iov.addr = (uint64_t)(dst - el->start + el->iov[id].addr);
    rma_iov.key = el->iov[id].key;
    if (rma_iov.key == 0) {
        DEBUG_MSG("\t No Key found for address %p on node %u", dst, id);
        return -1;
    }
    DEBUG_MSG("\t Writing %lu bytes from %p to address 0x%lx at node %u with key 0x%lx (threshold %lu, in-flight msgs: %lu)",
              size, src, rma_iov.addr, id, rma_iov.key, rofi.desc.inject_size,
              rofi.put_cntr);

    if (flags & ROFI_SYNC) {
        rofi_transport_put(&rofi, &rma_iov, id, src, size, el->mr_desc, NULL);
        rofi_transport_put_wait_all(&rofi);
    }
    else {
        rofi_transport_put(&rofi, &rma_iov, id, src, size, el->mr_desc, NULL);
    }
    return 0;
}

int rofi_get_internal(void *dst, void *src, size_t size, unsigned int id, unsigned long flags) {
    rofi_mr_desc *el = mr_get(dst);
    struct fi_rma_iov rma_iov;
    int ret = 0;

    if (!el) {
        DEBUG_MSG("\t No mr found for address %p on node %u", dst, id);
        return -1;
    }
    DEBUG_MSG("\t Found MR [0x%p - 0x%p] Key: 0x%lx", el->start, el->start + el->size, el->mr_key);

    rma_iov.addr = (uint64_t)(dst - el->start + el->iov[id].addr);
    rma_iov.key = el->iov[id].key;
    if (rma_iov.key == 0) {
        DEBUG_MSG("\t No Key found for address %p on node %u", dst, id);
        return -1;
    }
    DEBUG_MSG("\t Reading %lu bytes from address 0x%lx to %p at node %u with key 0x%lx (threshold %lu, in-flight msgs: %lu)",
              size, rma_iov.addr, src, id, rma_iov.key, rofi.desc.max_message_size,
              rofi.get_cntr);

    if (flags & ROFI_SYNC) {
        rofi_transport_get(&rofi, &rma_iov, id, dst, size, el->mr_desc, NULL);
        rofi_transport_get_wait_all(&rofi);
    }
    else {
        rofi_transport_get(&rofi, &rma_iov, id, dst, size, el->mr_desc, NULL);
    }
    return 0;
}

int rofi_send_internal(unsigned long id, void *buf, size_t size, unsigned long flags) {
}

int rofi_recv_internal(unsigned long id, void *buf, size_t size, unsigned long flags) {
}

int rofi_init_internal(char *prov) {
    if (!prov) {
        ERR_MSG("ROFI provider not specified. Currently ROFI only supports \"verbs\".");
    }
    int ret = 0;
    rofi.desc.PageSize = sysconf(_SC_PAGESIZE);

    ret = rt_init();
    if (ret) {
        ERR_MSG("Error initializing ROFI RT. Aborting.");
        goto err;
    }

    rofi.desc.nodes = rt_get_size();
    rofi.desc.nid = rt_get_rank();

    rofi.info = (struct fi_info *)calloc(1, sizeof(struct fi_info));
    if (!rofi.info) {
        ERR_MSG("Error allocating memory for rofi. Aborting.");
        ret = EXIT_FAILURE;
        goto err;
    }

    DEBUG_MSG("Initializing process %d/%d...", rofi.desc.nid, rofi.desc.nodes);

    struct fi_info *hints = fi_allocinfo();
    if (!hints) {
        return EXIT_FAILURE;
    }

    hints->caps = FI_RMA; // eventually want FI_ATOMIC
    hints->addr_format = FI_FORMAT_UNSPEC;
    hints->domain_attr->resource_mgmt = FI_RM_ENABLED;
    hints->domain_attr->threading = FI_THREAD_DOMAIN;
    hints->domain_attr->data_progress = FI_PROGRESS_AUTO;
    // FI_MR_BASIC == FI_MR_ALLOCATED | FI_MR_PROV_KEY | FI_MR_VIRT_ADDR -- which is only mode supported by verbs
    hints->domain_attr->mr_mode = FI_MR_ALLOCATED | FI_MR_PROV_KEY | FI_MR_VIRT_ADDR;
    hints->mode = FI_CONTEXT;
    hints->ep_attr->type = FI_EP_RDM;
    hints->tx_attr->op_flags = FI_DELIVERY_COMPLETE; // maybe need to change this to FI_INJECT_COMPLETE or FI_TRANSMIT_COMPLETE

    if (prov) {
        hints->fabric_attr->prov_name = strdup(prov);
    }

    // this isn't really needed for verbs since it is a connected endpoint
    rofi.remote_addrs = (fi_addr_t *)malloc(rofi.desc.nodes * sizeof(fi_addr_t));
    if (!rofi.remote_addrs) {
        ERR_MSG("Error allocating memory for remote addresses. Aborting!");
        return -ENOMEM;
    }

    for (int i = 0; i < rofi.desc.nodes; i++){
        rofi.remote_addrs[i] = i;
    }

    rofi_transport_init(hints, &rofi);

    mr_init();
    int rofi_alloc_exchange_size = rofi.desc.nodes * (sizeof(struct fi_rma_iov) + sizeof(uint64_t) * 3);

    rofi.alloc_mr = mr_add(&rofi, rofi_alloc_exchange_size, 0);
    if (!rofi.alloc_mr) {
        ERR_MSG("Error allocating memory for memory region alloc buffer. Aborting!");
        return -ENOMEM;
    }

    ret = rofi_transport_exchange_mr_info(&rofi, rofi.alloc_mr);

    if (ret) {
        return ret;
    }

    rofi.alloc_bufs.iov_buf = rofi.alloc_mr->start;
    rofi.alloc_bufs.hash_buf = rofi.alloc_mr->start + rofi.desc.nodes * sizeof(struct fi_rma_iov);
    rofi.alloc_bufs.barrier_buf = rofi.alloc_mr->start + rofi.desc.nodes * (sizeof(struct fi_rma_iov) + sizeof(uint64_t) * 2);
    rofi.alloc_bufs.barrier_id = 0;

    for (int i = 0; i < rofi.desc.nid; i++) {
        rofi.alloc_bufs.iov_buf[i].key = 0;
        rofi.alloc_bufs.iov_buf[i].addr = 0;
        rofi.alloc_bufs.hash_buf[i * 2] = 0;
        rofi.alloc_bufs.hash_buf[i * 2 + 1] = 0;
        rofi.alloc_bufs.barrier_buf[i] = 0;
    }
    fi_freeinfo(hints);
    return 0;

err:
    rofi.desc.status = ROFI_STATUS_ERR;
    return -1;
}

int rofi_finit_internal(void) {
    rofi.desc.status = ROFI_STATUS_TERM;
    rofi_wait_internal();

    if (rofi.desc.nodes > 1) {
        rt_barrier();
    }
    mr_free();

    rofi_transport_fini(&rofi);


    return 0;
}


