#include <assert.h>
#include <inttypes.h>
#include <math.h>
#include <rdma/fi_rma.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>
#include <rdma/fi_atomic.h>

#include <rofi.h>
#include <rofi_atomic.h>
#include <rofi_debug.h>
#include <rofi_internal.h>
#include <transport.h>

rofi_transport_t rofi = {0};

static int rofi_to_fi_datatype(rofi_datatype_t datatype, enum fi_datatype *fi_datatype) {
    switch (datatype) {
        case ROFI_DATATYPE_INT8:
            *fi_datatype = FI_INT8;
            return 0;
        case ROFI_DATATYPE_UINT8:
            *fi_datatype = FI_UINT8;
            return 0;
        case ROFI_DATATYPE_INT16:
            *fi_datatype = FI_INT16;
            return 0;
        case ROFI_DATATYPE_UINT16:
            *fi_datatype = FI_UINT16;
            return 0;
        case ROFI_DATATYPE_INT32:
            *fi_datatype = FI_INT32;
            return 0;
        case ROFI_DATATYPE_UINT32:
            *fi_datatype = FI_UINT32;
            return 0;
        case ROFI_DATATYPE_INT64:
            *fi_datatype = FI_INT64;
            return 0;
        case ROFI_DATATYPE_UINT64:
            *fi_datatype = FI_UINT64;
            return 0;
        case ROFI_DATATYPE_FLOAT:
            *fi_datatype = FI_FLOAT;
            return 0;
        case ROFI_DATATYPE_DOUBLE:
            *fi_datatype = FI_DOUBLE;
            return 0;
        case ROFI_DATATYPE_FLOAT_COMPLEX:
            *fi_datatype = FI_FLOAT_COMPLEX;
            return 0;
        case ROFI_DATATYPE_DOUBLE_COMPLEX:
            *fi_datatype = FI_DOUBLE_COMPLEX;
            return 0;
        default:
            return -1;
    }
}

static int rofi_to_fi_atomic_op(rofi_atomic_op_t op, enum fi_op *fi_op) {
    switch (op) {
        case ROFI_ATOMIC_OP_MIN:
            *fi_op = FI_MIN;
            return 0;
        case ROFI_ATOMIC_OP_MAX:
            *fi_op = FI_MAX;
            return 0;
        case ROFI_ATOMIC_OP_SUM:
            *fi_op = FI_SUM;
            return 0;
        case ROFI_ATOMIC_OP_PROD:
            *fi_op = FI_PROD;
            return 0;
        case ROFI_ATOMIC_OP_LOR:
            *fi_op = FI_LOR;
            return 0;
        case ROFI_ATOMIC_OP_LAND:
            *fi_op = FI_LAND;
            return 0;
        case ROFI_ATOMIC_OP_BOR:
            *fi_op = FI_BOR;
            return 0;
        case ROFI_ATOMIC_OP_BAND:
            *fi_op = FI_BAND;
            return 0;
        case ROFI_ATOMIC_OP_LXOR:
            *fi_op = FI_LXOR;
            return 0;
        case ROFI_ATOMIC_OP_BXOR:
            *fi_op = FI_BXOR;
            return 0;
        case ROFI_ATOMIC_OP_READ:
            *fi_op = FI_ATOMIC_READ;
            return 0;
        case ROFI_ATOMIC_OP_CSWAP:
            *fi_op = FI_CSWAP;
            return 0;
        case ROFI_ATOMIC_OP_CSWAP_NE:
            *fi_op = FI_CSWAP_NE;
            return 0;
        case ROFI_ATOMIC_OP_CSWAP_LE:
            *fi_op = FI_CSWAP_LE;
            return 0;
        case ROFI_ATOMIC_OP_CSWAP_LT:
            *fi_op = FI_CSWAP_LT;
            return 0;
        case ROFI_ATOMIC_OP_CSWAP_GE:
            *fi_op = FI_CSWAP_GE;
            return 0;
        case ROFI_ATOMIC_OP_CSWAP_GT:
            *fi_op = FI_CSWAP_GT;
            return 0;
        case ROFI_ATOMIC_OP_MSWAP:
            *fi_op = FI_MSWAP;
            return 0;
        case ROFI_ATOMIC_OP_WRITE:
            *fi_op = FI_ATOMIC_WRITE;
            return 0;
        default:
            return -1;
    }
}

static int rofi_query_atomic_internal_with_mode(rofi_datatype_t datatype, rofi_atomic_op_t op, int mode) {
    enum fi_datatype fi_datatype;
    enum fi_op fi_op;
    size_t count = 0;
    int ret;

    if (rofi_to_fi_datatype(datatype, &fi_datatype) != 0 || rofi_to_fi_atomic_op(op, &fi_op) != 0) {
        ERR_MSG("Invalid ROFI atomic query datatype (%d) or op (%d)", datatype, op);
        return -1;
    }

    if (!(rofi.info->caps & FI_ATOMIC) || rofi.domain == NULL || rofi.ep == NULL) {
        return -1;
    }

    switch (mode) {
        case 0:
            ret = fi_atomicvalid(rofi.ep, fi_datatype, fi_op, &count);
            break;
        case 1:
            ret = fi_fetch_atomicvalid(rofi.ep, fi_datatype, fi_op, &count);
            break;
        case 2:
            ret = fi_compare_atomicvalid(rofi.ep, fi_datatype, fi_op, &count);
            break;
        default:
            return -1;
    }

    if (ret) {
        return ret;
    }

    if (count < 1) {
        return -1;
    }

    return 0;
}

int rofi_query_atomic_internal(rofi_datatype_t datatype, rofi_atomic_op_t op) {
    return rofi_query_atomic_internal_with_mode(datatype, op, 0);
}

int rofi_query_fetch_atomic_internal(rofi_datatype_t datatype, rofi_atomic_op_t op) {
    return rofi_query_atomic_internal_with_mode(datatype, op, 1);
}

int rofi_query_compare_atomic_internal(rofi_datatype_t datatype, rofi_atomic_op_t op) {
    return rofi_query_atomic_internal_with_mode(datatype, op, 2);
}

int rofi_has_atomics_internal(void) {
    if (rofi.info == NULL || rofi.domain == NULL) {
        return 0;
    }

    return (rofi.info->caps & FI_ATOMIC) ? 1 : 0;
}

void *rofi_get_remote_addr_internal(void *addr, unsigned int id) {
    rofi_mr_desc *el = mr_get(&rofi, addr);
    int ret = 0;

    if (!el) {
        ERR_MSG("MR not found for address %p", addr);
        return NULL;
    }

    DEBUG_MSG("\t Found MR [0x%lx - 0x%lx] Addr: %p Key: 0x%lx ", el->start, el->start + el->size, (void *)(addr - (uintptr_t)el->start + el->iov[id].addr), el->iov[id].key);
    return (void *)(addr - (uintptr_t)el->start + el->iov[id].addr);
}

void *rofi_get_local_addr_from_remote_addr_internal(void *addr, unsigned int id) {
    rofi_mr_desc *el = mr_get_from_remote(&rofi, addr, id);
    int ret = 0;

    if (!el) {
        ERR_MSG("MR not found for remote address %p", addr);
        return NULL;
    }

    DEBUG_MSG("\t Found MR [0x%lx - 0x%lx] Addr: %p Key: 0x%lx", el->start, el->start + el->size, (void *)(addr - el->iov[id].addr + (uintptr_t)el->start), el->iov[id].key);
    return (void *)(addr - el->iov[id].addr + (uintptr_t)el->start);
}

int rofi_wait_internal(void) {
    rofi_transport_put_wait_all(&rofi);
    rofi_transport_get_wait_all(&rofi);
    // rofi_transport_barrier(&rofi);
    return 0;
}

void *rofi_alloc_internal(size_t size, unsigned long flags) {

    rofi_mr_desc *mr = mr_add(&rofi, size, flags);
    if (!mr) {
        ERR_MSG("Error allocating memory for memory region descriptor. Aborting!");
        return NULL;
    }

    if (rofi_transport_exchange_mr_info(&rofi, mr)) {
        ERR_MSG("Error exchanging memory region info. Aborting!");
        return NULL;
    }

    return mr->start;
}

void *rofi_sub_alloc_internal(size_t size, unsigned long flags, uint64_t *pes, uint64_t num_pes) {
    rofi_mr_desc *mr = mr_add(&rofi, size, flags);
    if (!mr) {
        ERR_MSG("Error allocating memory for memory region descriptor. Aborting!");
        return NULL;
    }

    if (rofi_transport_sub_exchange_mr_info(&rofi, mr, pes, num_pes)) {
        ERR_MSG("Error exchanging memory region info. Aborting!");
        return NULL;
    }

    return mr->start;
}

int rofi_release_internal(void *addr) {
    return mr_rm(&rofi, addr);
}

int rofi_sub_release_internal(void *addr, uint64_t *pes, uint64_t num_pes) {
    return mr_rm(&rofi, addr);
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
    // DEBUG_MSG("\t Flushing...");
    pthread_mutex_lock(&rofi.lock);
    rofi_transport_progress(&rofi);
    pthread_mutex_unlock(&rofi.lock);
    return 0;
}

void rofi_barrier_internal(void) {
    rofi_transport_barrier(&rofi);
}

int rofi_put_internal(void *dst, void *src, size_t size, unsigned int id, unsigned long flags) {
    rofi_mr_desc *el = mr_get(&rofi, dst);
    struct fi_rma_iov rma_iov;
    int ret = 0;

    if (!el) {
        ERR_MSG("\t No mr found for address %p on node %u", dst, id);
        return -1;
    }
    DEBUG_MSG("\t Found MR [0x%p - 0x%p] Key: 0x%lx for dst address %p", el->start, el->start + el->size, el->mr_key, dst);

    for (int i = 0; i < rofi.desc.nodes; i++) {
        DEBUG_MSG("remote addr: %d %p", i, el->iov[id].addr);
    }

#ifdef __OFI_PROV_CXI__
    // if CXI, use offset instead of virtual address
    rma_iov.addr = (uint64_t)(dst - el->start + el->iov[id].addr) - (uint64_t)el->start;
#else
    rma_iov.addr = (uint64_t)(dst - el->start + el->iov[id].addr);
#endif
    rma_iov.key = el->iov[id].key;
    if (rma_iov.key == 0) {
        ERR_MSG("\t No Key found for address %p on node %u", dst, id);
        return -1;
    }
    DEBUG_MSG("\t Writing %lu bytes from %p to address 0x%lx at node %u with key 0x%lx (threshold %lu, in-flight msgs: %lu) sync: %d",
              size, src, rma_iov.addr, id, rma_iov.key, rofi.desc.inject_size,
              rofi.put_cntr,
              flags & ROFI_SYNC);

    
    if (flags & ROFI_SYNC) {
        if (rofi_transport_put(&rofi, &rma_iov, id, src, size, el->mr_desc, NULL)) {
            ERR_MSG("\t Error writing %lu bytes from %p to address 0x%lx at node %u with key 0x%lx",
                    size, src, rma_iov.addr, id, rma_iov.key);
            return -1;
        }
        if (rofi_transport_put_wait_all(&rofi)) {
            ERR_MSG("\t Error waiting for put");
            return -1;
        }
    }
    else {
        if (rofi_transport_put(&rofi, &rma_iov, id, src, size, el->mr_desc, NULL)) {
            ERR_MSG("\t Error writing %lu bytes from %p to address 0x%lx at node %u with key 0x%lx",
                    size, src, rma_iov.addr, id, rma_iov.key);
            return -1;
        }
    }
    DEBUG_MSG("\t Done writing %lu bytes from %p to address 0x%lx at node %u with key 0x%lx",
              size, src, rma_iov.addr, id, rma_iov.key);
    return 0;
}

int rofi_get_internal(void *dst, void *src, size_t size, unsigned int id, unsigned long flags) {
    rofi_mr_desc *el = mr_get(&rofi, src);
    struct fi_rma_iov rma_iov;
    int ret = 0;

    if (!el) {
        ERR_MSG("\t No mr found for address %p on node %u", src, id);
        return -1;
    }
    DEBUG_MSG("\t Found MR [0x%p - 0x%p] Key: 0x%lx", el->start, el->start + el->size, el->mr_key);

#ifdef __OFI_PROV_CXI__
    rma_iov.addr = (uint64_t)(src - el->start + el->iov[id].addr) - (uint64_t)el->start;
#else
    rma_iov.addr = (uint64_t)(src - el->start + el->iov[id].addr);
#endif
    rma_iov.key = el->iov[id].key;
    if (rma_iov.key == 0) {
        ERR_MSG("\t No Key found for address %p on node %u", src, id);
        return -1;
    }
    DEBUG_MSG("\t Reading %lu bytes from address 0x%lx to %p at node %u with key 0x%lx (threshold %lu, in-flight msgs: %lu) sync: %d",
              size, rma_iov.addr, dst, id, rma_iov.key, rofi.desc.max_message_size,
              rofi.get_cntr,
              flags & ROFI_SYNC);

    if (flags & ROFI_SYNC) {

        if (rofi_transport_get(&rofi, &rma_iov, id, dst, size, el->mr_desc, NULL)) {
            ERR_MSG("\t Error reading %lu bytes from address 0x%lx to %p at node %u with key 0x%lx",
                    size, rma_iov.addr, src, id, rma_iov.key);
            return -1;
        }
        if (rofi_transport_get_wait_all(&rofi)) {
            ERR_MSG("\t Error waiting for get");
        }
    }
    else {
        if (rofi_transport_get(&rofi, &rma_iov, id, dst, size, el->mr_desc, NULL)) {
            ERR_MSG("\t Error reading %lu bytes from address 0x%lx to %p at node %u with key 0x%lx",
                    size, rma_iov.addr, src, id, rma_iov.key);
            return -1;
        }
    }
    DEBUG_MSG("\t Done reading %lu bytes from address 0x%lx to %p at node %u with key 0x%lx",
              size, rma_iov.addr, dst, id, rma_iov.key);
    return 0;
}

int rofi_send_internal(unsigned int pe, void *buf, size_t size, unsigned long flags) {
    if (rofi_transport_send(&rofi, buf, size, pe)) {
        ERR_MSG("\t Error sending %lu bytes to node %u", size, pe);
        return -1;
    }
    return 0;
}

int rofi_recv_internal(void *buf, size_t size, unsigned long flags) {
    if (rofi_transport_recv(&rofi, buf, size)) {
        ERR_MSG("\t Error receiving %lu bytes", size);
        return -1;
    }
    return 0;
}

static int rofi_prepare_atomic_iov(void *addr, unsigned int id, struct fi_rma_iov *rma_iov) {
    rofi_mr_desc *el = mr_get(&rofi, addr);

    if (!el) {
        ERR_MSG("MR not found for address %p on node %u", addr, id);
        return -1;
    }
    DEBUG_MSG("\t Found MR [0x%p - 0x%p] Key: 0x%lx for dst address %p", el->start, el->start + el->size, el->mr_key, addr);

    rma_iov->addr = (uint64_t)((uintptr_t)addr - (uintptr_t)el->start + el->iov[id].addr);
    if (rma_iov->addr == 0) {
        ERR_MSG("\t No address found for address %p on node %u", addr, id);
        return -1;
    }

    rma_iov->key = el->iov[id].key;
    if (rma_iov->key == 0) {
        ERR_MSG("\t No Key found for address %p on node %u", addr, id);
        return -1;
    }

    return 0;
}

static void *rofi_get_local_mr_desc(const void *addr) {
    rofi_mr_desc *el;

    if (addr == NULL) {
        return NULL;
    }

    el = mr_get(&rofi, (void *)addr);
    if (!el) {
        return NULL;
    }

    return el->mr_desc;
}

ssize_t rofi_atomic_op_internal(void *addr, const void *value, size_t count, rofi_datatype_t datatype, rofi_atomic_op_t op,
                                unsigned int id) {
    enum fi_datatype fi_datatype;
    enum fi_op fi_op;
    struct fi_rma_iov rma_iov;
    void *value_desc;
    ssize_t ret;

    if (addr == NULL || value == NULL || count < 1 || id >= rofi.desc.nodes) {
        ERR_MSG("Invalid atomic_op arguments. addr=%p value=%p count=%lu id=%u", addr, value, count, id);
        return -1;
    }

    if (rofi_to_fi_datatype(datatype, &fi_datatype) != 0 || rofi_to_fi_atomic_op(op, &fi_op) != 0) {
        ERR_MSG("Invalid atomic_op datatype (%d) or op (%d)", datatype, op);
        return -1;
    }

    if (rofi_query_atomic_internal(datatype, op) != 0) {
        ERR_MSG("Atomic op is not supported by the provider for datatype (%d) and op (%d)", datatype, op);
        return -1;
    }

    if (rofi_prepare_atomic_iov(addr, id, &rma_iov) != 0) {
        return -1;
    }

    value_desc = rofi_get_local_mr_desc(value);

    ret = rofi_transport_atomic(&rofi, &rma_iov, id, value, count, fi_datatype, fi_op, value_desc, NULL);
    return ret;
}

ssize_t rofi_atomic_fetch_internal(void *addr, const void *value, void *result, size_t count, rofi_datatype_t datatype,
                                   rofi_atomic_op_t op, unsigned int id) {
    enum fi_datatype fi_datatype;
    enum fi_op fi_op;
    struct fi_rma_iov rma_iov;
    void *value_desc;
    void *result_desc;
    ssize_t ret;

    if (addr == NULL || result == NULL || count < 1 || id >= rofi.desc.nodes) {
        ERR_MSG("Invalid atomic_fetch arguments. addr=%p result=%p count=%lu id=%u", addr, result, count, id);
        return -1;
    }

    if (rofi_to_fi_datatype(datatype, &fi_datatype) != 0 || rofi_to_fi_atomic_op(op, &fi_op) != 0) {
        ERR_MSG("Invalid atomic_fetch datatype (%d) or op (%d)", datatype, op);
        return -1;
    }

    if (rofi_query_fetch_atomic_internal(datatype, op) != 0) {
        ERR_MSG("Atomic fetch is not supported by the provider for datatype (%d) and op (%d)", datatype, op);
        return -1;
    }

    if (rofi_prepare_atomic_iov(addr, id, &rma_iov) != 0) {
        return -1;
    }

    value_desc = rofi_get_local_mr_desc(value);
    result_desc = rofi_get_local_mr_desc(result);

    ret = rofi_transport_atomic_fetch(&rofi, &rma_iov, id, value, result, count, fi_datatype, fi_op, value_desc,
                                      result_desc, NULL);
    return ret;
}

ssize_t rofi_compare_atomic_internal(void *addr, const void *value, const void *compare, void *result, size_t count,
                                     rofi_datatype_t datatype, rofi_atomic_op_t op, unsigned int id) {
    enum fi_datatype fi_datatype;
    enum fi_op fi_op;
    struct fi_rma_iov rma_iov;
    void *value_desc;
    void *compare_desc;
    void *result_desc;
    ssize_t ret;

    if (addr == NULL || value == NULL || compare == NULL || result == NULL || count < 1 || id >= rofi.desc.nodes) {
        ERR_MSG("Invalid compare_atomic arguments. addr=%p value=%p compare=%p result=%p count=%lu id=%u", addr,
                value, compare, result, count, id);
        return -1;
    }

    if (rofi_to_fi_datatype(datatype, &fi_datatype) != 0 || rofi_to_fi_atomic_op(op, &fi_op) != 0) {
        ERR_MSG("Invalid compare_atomic datatype (%d) or op (%d)", datatype, op);
        return -1;
    }

    if (rofi_query_compare_atomic_internal(datatype, op) != 0) {
        ERR_MSG("Compare atomic is not supported by the provider for datatype (%d) and op (%d)", datatype, op);
        return -1;
    }

    if (rofi_prepare_atomic_iov(addr, id, &rma_iov) != 0) {
        return -1;
    }

    value_desc = rofi_get_local_mr_desc(value);
    compare_desc = rofi_get_local_mr_desc(compare);
    result_desc = rofi_get_local_mr_desc(result);

    ret = rofi_transport_compare_atomic(&rofi, &rma_iov, id, value, compare, result, count, fi_datatype, fi_op,
                                        value_desc, compare_desc, result_desc, NULL);
    return ret;
}

rofi_names_t *rofi_parse_names_internal(char *names_list) {
    char token = ';';
    int name_cnt = 0;
    for (int i = 0; i < strlen(names_list); i++) {
        if (names_list[i] == token) {
            name_cnt++;
        }
    }
    name_cnt += 1;
    char **name_strs = (char **)calloc(name_cnt, sizeof(char *));

    int p = 0;
    int i = 0;
    for (int k = 0; k < strlen(names_list); k++) {
        if (names_list[k] == token) {
            name_strs[p] = strndup(&names_list[i], k - i);
            p++;
            i = k + 1;
        }
    }
    name_strs[p] = strndup(&names_list[i], strlen(names_list) - i);
    rofi_names_t *names = (rofi_names_t *)calloc(1, sizeof(rofi_names_t));
    names->num = name_cnt;
    names->names = name_strs;
    return names;
}



int rofi_init_internal(char *provs, char *domains) {
    pthread_rwlock_init(&rofi.mr_lock, NULL);
    pthread_mutex_init(&rofi.lock, NULL);
    int ret = 0;
    rofi.desc.PageSize = sysconf(_SC_PAGESIZE);

    ret = rt_init();
    if (ret) {
        ERR_MSG("Error initializing ROFI RT. Aborting.");
        goto err;
    }

    rofi.desc.nodes = rt_get_size();
    rofi.desc.nid = rt_get_rank();

    rofi.info = NULL;

    DEBUG_MSG("Initializing process %d/%d...", rofi.desc.nid, rofi.desc.nodes);

    struct fi_info *hints = fi_allocinfo();
    if (!hints) {
        return EXIT_FAILURE;
    }

#if defined (__OFI_PROV_VERBS__)

    hints->caps = FI_RMA | FI_ATOMIC | FI_COLLECTIVE | FI_MSG;
    hints->addr_format = FI_FORMAT_UNSPEC;
    hints->domain_attr->resource_mgmt = FI_RM_ENABLED;
    hints->domain_attr->threading = FI_THREAD_DOMAIN;
    hints->domain_attr->data_progress = FI_PROGRESS_MANUAL;
    hints->domain_attr->mr_mode = FI_MR_BASIC; // FI_MR_ALLOCATED | FI_MR_PROV_KEY | FI_MR_VIRT_ADDR; //we do FI_MR_BASIC because tcp will clear the individual flags thus would require us to make our mr offsets 0-based
    hints->mode = FI_CONTEXT;
    hints->ep_attr->type = FI_EP_RDM;
    hints->tx_attr->op_flags = FI_DELIVERY_COMPLETE; // maybe need to change this to FI_INJECT_COMPLETE or FI_TRANSMIT_COMPLETE

#elif defined (__OFI_PROV_CXI__)

    DEBUG_MSG("Process %d requesting CXI provider", rofi.desc.nid);

    // NOTES: 
    // (1) The following hints are based on working through the code path 
    // for the simple_write rma test included with the CXI provider in 
    // libfabric v2.1
    // (2) The verbs provider specifies FI_DELIVERY_COMPLETE; CXI supports this, but defaults to 
    // FI_TRANSMIT_COMPLETE, as it has lower latency. TRANSMIT vs DELIVERY does not make 
    // a difference for passing the ROFI tests. 
    // TODO: Will TRANSMIT break lamelar?
    // (3) hints->tx_attr->size defaults to 1024, and is a hard cap (i.e., will abort 
    // if exceeded). A large fan-out could cause issues. TODO: What is a safe value?
    // (4) CXI has 'optimized' memory regions for applications that will use a small 
    // number of large regions involving many small operations. Enabling these requires 
    // manually selecting memory keys in the range 0-99 inclusive. This 
    // in turn requires not configuring with FI_MR_PROV_KEY. Making this change may 
    // cause ripple effects across the ROFI code. TODO: Consider whether to take this on.
    // (6) TODO: Explore message ordering constraints. Currently we leave them at the defaults.
    // (7) TODO: The CXI provider is returned with all of its capabilites active (including 
    // FI_RMA_EVENT), which is a superset of the caps requested for the verbs provider. Test 
    // In other work we have not seen a performance impact of doing this. 

    hints->fabric_attr->prov_name = strdup("cxi"); // limits returned providers to cxi only
    hints->domain_attr->mr_mode = FI_MR_ENDPOINT | FI_MR_ALLOCATED | FI_MR_PROV_KEY; 
    hints->domain_attr->data_progress = FI_PROGRESS_MANUAL;
    hints->domain_attr->control_progress = FI_PROGRESS_MANUAL; 
    hints->tx_attr->size = 4096;  
    //hints->tx_attr->op_flags = FI_DELIVERY_COMPLETE;  // default for CXI is FI_TRANSMIT_COMPLETE

    // TBD: message ordering requirements (tx_attr and rx_attr)
    // Through trial and error, we determined This will make most of them NO;
    // hints->tx_attr->msg_order = 0;

    DEBUG_MSG("ROFI Compiled for CXI support");

#else
    ERR_MSG("Invalid or no OFI provider selected during ROFI configuration!");
#endif

    rofi_names_t *prov_names = NULL;
    if (provs) {
        prov_names = rofi_parse_names_internal(provs);
    }

    rofi_names_t *domain_names = NULL;
    if (domains) {
        domain_names = rofi_parse_names_internal(domains);
    }

    // I think the endpoints we support are all connected so I'm not sure these are even used?
    rofi.remote_addrs = (fi_addr_t *)malloc(rofi.desc.nodes * sizeof(fi_addr_t));
    if (!rofi.remote_addrs) {
        ERR_MSG("Error allocating memory for remote addresses. Aborting!");
        return -ENOMEM;
    }

    for (int i = 0; i < rofi.desc.nodes; i++) {
        rofi.remote_addrs[i] = i;
    }

    rofi_transport_init(hints, &rofi, prov_names, domain_names);

    if (prov_names) {
        for (int i = 0; i < prov_names->num; i++) {
            free(prov_names->names[i]);
        }
        free(prov_names->names);
        free(prov_names);
    }

    if (domain_names) {
        for (int i = 0; i < domain_names->num; i++) {
            free(domain_names->names[i]);
        }
        free(domain_names->names);
        free(domain_names);
    }

    mr_init();
    uint64_t global_barrier_size = rofi.desc.nodes * sizeof(uint64_t);
    uint64_t sub_alloc_barrier_size = rofi.desc.nodes * sizeof(uint64_t);
    uint64_t sub_alloc_size = rofi.desc.nodes * sizeof(struct fi_rma_iov);
    int rofi_mr_size = global_barrier_size + sub_alloc_barrier_size + sub_alloc_size;

    rofi.mr = mr_add(&rofi, rofi_mr_size, 0);
    if (!rofi.mr) {
        ERR_MSG("Error allocating memory for memory region alloc buffer. Aborting!");
        return -ENOMEM;
    }
    
    if (rofi.desc.nodes > 1) {
        rt_barrier();
    }

    ret = rofi_transport_exchange_init_mr_info(&rofi, rofi.mr);
    // ORIGINAL: ret = rofi_transport_exchange_mr_info(&rofi, rofi.mr);

    if (ret) {
        return ret;
    }

    rofi.global_barrier_id = 0;
    rofi.sub_alloc_barrier_id = 0;
    rofi.global_barrier_buf = (uint64_t *)rofi.mr->start;
    rofi.sub_alloc_barrier_buf = (uint64_t *)(rofi.mr->start + global_barrier_size);
    rofi.sub_alloc_buf = (struct fi_rma_iov *)(rofi.mr->start + global_barrier_size + sub_alloc_barrier_size);

    for (int i = 0; i < rofi.desc.nodes; i++) {
        rofi.global_barrier_buf[i] = 0;
        rofi.sub_alloc_barrier_buf[i] = 0;
        rofi.sub_alloc_buf[i].key = 0;
        rofi.sub_alloc_buf[i].addr = 0;
    }
    fi_freeinfo(hints);

    //if (rofi.desc.nodes > 1) {
    //    rt_barrier();
    //}
#ifdef __OFI_PROV_CXI__
    rofi_transport_barrier_msg(&rofi);
#else
    rofi_transport_barrier(&rofi);
#endif
    return 0;

err:
    rofi.desc.status = ROFI_STATUS_ERR;
    return -1;
}

int rofi_finit_internal(void) {
    DEBUG_MSG("rofi_finit_internal");
    rofi.desc.status = ROFI_STATUS_TERM;
    rofi_wait_internal();
    pthread_mutex_lock(&rofi.lock);
    rofi_transport_progress(&rofi);
    pthread_mutex_unlock(&rofi.lock);

    if (rofi.desc.nodes > 1) {
        rt_barrier();
    }
    mr_free(&rofi);

    pthread_mutex_lock(&rofi.lock);
    rofi_transport_fini(&rofi);
    pthread_mutex_unlock(&rofi.lock);
    if (rofi.desc.nodes > 1) {
        rt_barrier();
    }
    rt_finit();

    return 0;
}
