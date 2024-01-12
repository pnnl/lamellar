/*
 * Copyright (c) 2013-2018 Intel Corporation.  All rights reserved.
 * Copyright (c) 2016 Cray Inc.  All rights reserved.
 * Copyright (c) 2014-2017, Cisco Systems, Inc. All rights reserved.
 *
 * This software is available to you under the BSD license below:
 *
 *     Redistribution and use in source and binary forms, with or
 *     without modification, are permitted provided that the following
 *     conditions are met:
 *
 *      - Redistributions of source code must retain the above
 *        copyright notice, this list of conditions and the following
 *        disclaimer.
 *
 *      - Redistributions in binary form must reproduce the above
 *        copyright notice, this list of conditions and the following
 *        disclaimer in the documentation and/or other materials
 *        provided with the distribution.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
 * BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
 * ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

#include <assert.h>
#include <netdb.h>
#include <poll.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#include <rdma/fi_atomic.h>
#include <rdma/fi_cm.h>
#include <rdma/fi_collective.h>
#include <rdma/fi_domain.h>
#include <rdma/fi_endpoint.h>
#include <rdma/fi_errno.h>
#include <rdma/fi_rma.h>
#include <rdma/fi_tagged.h>

#include "rofi_debug.h"
#include "rofi_internal.h"
#include "transport.h"

int rofi_transport_fini(rofi_transport_t *rofi) {
    free(rofi->info);

    int ret = fi_close(&rofi->ep->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        rofi->ep = NULL;
    }

    ret = fi_close(&rofi->cq->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fl_close", ret);
        rofi->cq = NULL;
    }

    ret = fi_close(&rofi->put_cntr->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        rofi->put_cntr = NULL;
    }

    ret = fi_close(&rofi->get_cntr->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        rofi->get_cntr = NULL;
    }

    ret = fi_close(&rofi->av->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        rofi->av = NULL;
    }

    ret = fi_close(&rofi->domain->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        rofi->domain = NULL;
    }

    ret = fi_close(&rofi->fabric->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        rofi->fabric = NULL;
    }
}

int rofi_transport_init(struct fi_info *hints, rofi_transport_t *rofi) {
    int ret = fi_getinfo(ROFI_FI_VERSION, NULL, NULL, 0, hints, &rofi->info);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_getinfo", ret);
    }

    ret = rofi_transport_init_fabric_resources(rofi);
    if (ret) {
        // already would have printed the error.
        return ret;
    }

    if (strncmp(rofi->info->fabric_attr->prov_name, "verbs", 5)) {
        ERR_MSG(" Only 'verbs' fabric is supported. Aborting.");
        return -1;
    }

    DEBUG_MSG("\tSelected Provider: %s  Version: (%u.%u) Fabric: %s Domain: %s max_inject: %zu, max_msg: %zu stx: %s MR_RMA_EVENT: %s msg: %s rma: %s read: %s write: %s remote_read: %s remote_write: %s rma_event: %s src_addr %s src_addrlen %lu dest_addr %s dest_addrlen %lu",
              rofi->info->fabric_attr->prov_name,
              FI_MAJOR(rofi->info->fabric_attr->prov_version),
              FI_MINOR(rofi->info->fabric_attr->prov_version),
              rofi->info->fabric_attr->name,
              rofi->info->domain_attr->name,
              rofi->info->tx_attr->inject_size,
              rofi->info->ep_attr->max_msg_size,
              rofi->info->domain_attr->max_ep_stx_ctx == 0 ? "no" : "yes",
              rofi->info->domain_attr->mr_mode & FI_MR_RMA_EVENT ? "yes" : " no",
              rofi->info->caps & FI_MSG ? "yes" : "no",
              rofi->info->caps & FI_RMA ? "yes" : "no",
              rofi->info->caps & FI_READ ? "yes" : "no",
              rofi->info->caps & FI_WRITE ? "yes" : "no",
              rofi->info->caps & FI_REMOTE_READ ? "yes" : "no",
              rofi->info->caps & FI_REMOTE_WRITE ? "yes" : "no",
              rofi->info->caps & FI_RMA_EVENT ? "yes" : "no",
              rofi->info->src_addr, rofi->info->src_addrlen,
              rofi->info->dest_addr, rofi->info->dest_addrlen);

    rofi->desc.max_message_size = rofi->info->ep_attr->max_msg_size;
    rofi->desc.inject_size = rofi->info->tx_attr->inject_size;

    ret = rofi_transport_init_endpoint_resources(rofi);
    if (ret) {
        // already would have printed the error.
        return ret;
    }

    char epname[512];
    size_t len = 64;
    ret = fi_getname(&rofi->ep->fid, &epname, &len);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_getname", ret);
        return ret;
    }
    rt_put("epname_len", &len, sizeof(size_t));
    rt_put("epname", epname, len);
    rofi->desc.addrlen = len;
    rt_exchange();

    ret = rofi_transport_init_av(rofi);
    if (ret) {
        // already would have printed the error.
        return ret;
    }
    return 0;
}

int rofi_transport_init_fabric_resources(rofi_transport_t *rofi) {
    int ret = fi_fabric(rofi->info->fabric_attr, &rofi->fabric, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_fabric", ret);
        return ret;
    }
    ret = fi_domain(rofi->fabric, rofi->info, &rofi->domain, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_domain", ret);
        return ret;
    }

    struct fi_av_attr av_attr = {0};
    if (rofi->info->domain_attr->av_type != FI_AV_UNSPEC) {
        av_attr.type = rofi->info->domain_attr->av_type;
    }

    ret = fi_av_open(rofi->domain, &av_attr, &rofi->av, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_av_open", ret);
        return ret;
    }
    return 0;
}
int rofi_transport_init_endpoint_resources(rofi_transport_t *rofi) {
    struct fi_cntr_attr put_cntr_attr = {0};
    struct fi_cntr_attr get_cntr_attr = {0};
    put_cntr_attr.events = FI_CNTR_EVENTS_COMP;
    get_cntr_attr.events = FI_CNTR_EVENTS_COMP;
    put_cntr_attr.wait_obj = FI_WAIT_NONE;
    get_cntr_attr.wait_obj = FI_WAIT_NONE;

    int ret = fi_cntr_open(rofi->domain, &put_cntr_attr, &rofi->put_cntr, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_cntr_open", ret);
        return ret;
    }
    ret = fi_cntr_open(rofi->domain, &get_cntr_attr, &rofi->get_cntr, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_cntr_open", ret);
        return ret;
    }

    struct fi_cq_attr cq_attr = {0};
    cq_attr.wait_obj = FI_WAIT_NONE;
    cq_attr.format = FI_CQ_FORMAT_CONTEXT;

    ret = fi_cq_open(rofi->domain, &cq_attr, &rofi->cq, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_cq_open", ret);
        return ret;
    }

    rofi->info->ep_attr->tx_ctx_cnt = 0;
    rofi->info->caps = FI_RMA | FI_WRITE | FI_READ;
    rofi->info->tx_attr->op_flags = FI_DELIVERY_COMPLETE;
    rofi->info->mode = 0;
    rofi->info->tx_attr->mode = 0;
    rofi->info->rx_attr->mode = 0;
    rofi->info->tx_attr->caps = rofi->info->caps;
    rofi->info->rx_attr->caps = FI_RECV; // to drive progress (is this only requried in manual progress mode?)

    ret = fi_endpoint(rofi->domain, rofi->info, &rofi->ep, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_endpoint", ret);
        return ret;
    }

    // bind address vector
    ret = fi_ep_bind(rofi->ep, &rofi->av->fid, 0);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_ep_bind av", ret);
        return ret;
    }

    // bind put cntr
    ret = fi_ep_bind(rofi->ep, &rofi->put_cntr->fid, FI_WRITE);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_ep_bind put_cntr", ret);
        return ret;
    }

    // bind get cntr
    ret = fi_ep_bind(rofi->ep, &rofi->get_cntr->fid, FI_READ);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_ep_bind get_cntr", ret);
        return ret;
    }

    // bind cq -- use same completion queue for send and recv -- I think we can remove FI_RECV when not using manual progress mode
    ret = fi_ep_bind(rofi->ep, &rofi->cq->fid, FI_SELECTIVE_COMPLETION | FI_TRANSMIT | FI_RECV);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_ep_bind cq", ret);
        return ret;
    }

    ret = fi_enable(rofi->ep);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_enable", ret);
        return ret;
    }
    return ret;
}

int rofi_transport_init_av(rofi_transport_t *rofi) {
    char *all_addrs = (char *)malloc(rofi->desc.nodes * rofi->desc.addrlen);
    assert(all_addrs);

    for (int i = 0; i < rofi->desc.nodes; i++) {
        char *addr_ptr = all_addrs + i * rofi->desc.addrlen;
        int ret = rt_get(i, "epname", addr_ptr, rofi->desc.addrlen);
        if (ret) {
            ERR_MSG("Error getting EP address name from %i (%d).", i, ret);
            return -1;
        }
    }
    int ret = fi_av_insert(rofi->av, all_addrs, rofi->desc.nodes, rofi->remote_addrs, 0, NULL);
    if (ret < 0) {
        ROFI_TRANSPORT_ERR_MSG("ft_av_insert", ret);
        return ret;
    }
    else if (ret != rofi->desc.nodes) {
        ERR_MSG("fi_av_insert: number of addresses inserted = %d;"
                " number of addresses given = %d\n",
                ret, rofi->desc.nodes);
        return -1;
    }
    return 0;
}

// // only need this if we use MANUAL_PROGRESS
// void rofi_transport_progress(rofi_transport_t *rofi) {
//     struct fi_c_entry buf = {0};
//     int ret = fi_cq_read(rofi->cq, &buf, 1);
//     if (ret == 1) {
//         printf("unexpected cq event\n"); // warnd
//     }
// }

// checks the rma return status, and aborts if not -FI_EAGAIN
// note this does not try to make progress
void rofi_transport_ctx_check_err(rofi_transport_t *rofi, int err) {
    if (err == -FI_EAGAIN) {
        struct fi_cq_err_entry ebuf = {0};
        int ret = fi_cq_readerr(rofi->cq, (void *)&ebuf, 0);
        if (ret == 1) {
            const char *errmsg = fi_cq_strerror(rofi->cq, ebuf.prov_errno, ebuf.err_data, NULL, 0);
            ERR_MSG("Error: %s\n", errmsg);
            abort();
        }
        else if (ret && ret != -FI_EAGAIN) {
            ROFI_TRANSPORT_ERR_MSG("fi_cq_readerr", ret);
            abort();
        }
    }
    else if (err) {
        ROFI_TRANSPORT_ERR_MSG("", err);
        abort();
    }
}

// checks the rma return status, and aborts if not -FI_EAGAIN
// otherwise tries to make progress
void rofi_transport_check_rma_err(rofi_transport_t *rofi, int err) {
    if (err == -FI_EAGAIN) {
        struct fi_cq_err_entry ebuf = {0};
        int ret = fi_cq_readerr(rofi->cq, (void *)&ebuf, 0);
        if (ret == 1) {
            const char *errmsg = fi_cq_strerror(rofi->cq, ebuf.prov_errno, ebuf.err_data, NULL, 0);
            printf("Error: %s\n", errmsg);
            abort();
        }
        else if (ret && ret != -FI_EAGAIN) {
            ROFI_TRANSPORT_ERR_MSG("fi_cq_readerr", ret);
            abort();
        }
        // rofi_transport_progress();
    }
    else if (err) {
        ROFI_TRANSPORT_ERR_MSG("", err);
        abort();
    }
}

void rofi_transport_put_inject(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, fi_addr_t pe, const void *src_addr, size_t len) {
    rofi->pending_put_cntr += 1;
    DEBUG_MSG("fi_inject_write %p %p %d %d %p 0x%lx", rofi->ep, src_addr, len, pe, rma_iov->addr, rma_iov->key);
    int ret = fi_inject_write(rofi->ep, src_addr, len, pe, rma_iov->addr, rma_iov->key);
    while (ret) { // retry while FI_EAGAIN
        rofi_transport_check_rma_err(rofi, ret);
        ret = fi_inject_write(rofi->ep, src_addr, len, pe, rma_iov->addr, rma_iov->key);
    }
}

void rofi_transport_put_large(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, fi_addr_t pe, const void *src_addr, size_t len, void *desc, void *context) {

    uint8_t *src_cur_addr = (uint8_t *)src_addr;
    uint8_t *src_end_addr = src_cur_addr + len;
    uint64_t dst_cur_addr = (uint64_t)rma_iov->addr;

    while (src_cur_addr < src_end_addr) {
        uint64_t cur_len = MIN(src_end_addr - src_cur_addr, rofi->desc.max_message_size);
        rofi->pending_put_cntr += 1;
        int ret = fi_write(rofi->ep, src_cur_addr, cur_len, desc, pe, dst_cur_addr, rma_iov->key, context);
        while (ret) { // retry while FI_EAGAIN
            rofi_transport_check_rma_err(rofi, ret);
            ret = fi_write(rofi->ep, src_cur_addr, cur_len, desc, pe, dst_cur_addr, rma_iov->key, context);
        }
        src_cur_addr += cur_len;
        dst_cur_addr += cur_len;
    }
}

// for PE need to check if using FI_AV_MAP, and then index into that
void rofi_transport_put(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, const void *src_addr, size_t len, void *desc, void *context) {
    if (len < rofi->desc.inject_size) {
        rofi_transport_put_inject(rofi, rma_iov, rofi->remote_addrs[pe], src_addr, len);
    }
    else {
        rofi_transport_put_large(rofi, rma_iov, rofi->remote_addrs[pe], src_addr, len, desc, context);
    }
}

void rofi_transport_put_wait_all(rofi_transport_t *rofi) {
    uint64_t prev_cnt = 0;
    while (prev_cnt < rofi->pending_put_cntr) { // essentially ensure no new puts came in
        int ret = fi_cntr_wait(rofi->put_cntr, prev_cnt, -1);
        prev_cnt = rofi->pending_put_cntr;
        rofi_transport_ctx_check_err(rofi, ret);
    }
    assert(rofi->pending_put_cntr == prev_cnt);
}

void rofi_transport_get_small(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, void *dst_addr, size_t len, void *desc, void *context) {
    rofi->pending_get_cntr += 1;
    int ret = fi_read(rofi->ep, dst_addr, len, desc, pe, rma_iov->addr, rma_iov->key, context);
    while (ret) { // retry while FI_EAGAIN
        rofi_transport_check_rma_err(rofi, ret);
        ret = fi_read(rofi->ep, dst_addr, len, desc, pe, rma_iov->addr, rma_iov->key, context);
    }
}

void rofi_transport_get_large(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, void *dst_addr, size_t len, void *desc, void *context) {
    uint64_t src_cur_addr = (uint64_t)rma_iov->addr;

    uint8_t *dst_cur_addr = (uint8_t *)dst_addr;
    uint8_t *dst_end_addr = dst_cur_addr + len;
    while (dst_cur_addr < dst_end_addr) {
        uint64_t cur_len = MIN(dst_end_addr - dst_cur_addr, rofi->desc.max_message_size);
        rofi->pending_get_cntr += 1;
        int ret = fi_read(rofi->ep, dst_cur_addr, cur_len, desc, pe, src_cur_addr, rma_iov->key, context);
        while (ret) { // retry while FI_EAGAIN
            rofi_transport_check_rma_err(rofi, ret);
            ret = fi_read(rofi->ep, dst_cur_addr, cur_len, desc, pe, src_cur_addr, rma_iov->key, context);
        }
        src_cur_addr += cur_len;
        dst_cur_addr += cur_len;
    }
}

void rofi_transport_get(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, void *dst_addr, size_t len, void *desc, void *context) {
    if (len < rofi->desc.max_message_size) {
        rofi_transport_get_small(rofi, rma_iov, rofi->remote_addrs[pe], dst_addr, len, desc, context);
    }
    else {
        rofi_transport_get_large(rofi, rma_iov, rofi->remote_addrs[pe], dst_addr, len, desc, context);
    }
}

void rofi_transport_get_wait_all(rofi_transport_t *rofi) {
    uint64_t prev_cnt = 0;
    while (prev_cnt < rofi->pending_get_cntr) { // essentially ensure no new gets came in
        int ret = fi_cntr_wait(rofi->get_cntr, prev_cnt, -1);
        prev_cnt = rofi->pending_get_cntr;
        rofi_transport_ctx_check_err(rofi, ret);
    }
    assert(rofi->pending_get_cntr == prev_cnt);
}

int rofi_transport_exchange_mr_info(rofi_transport_t *rofi, rofi_mr_desc *mr) {
    if (rofi->desc.nodes == 1) {
        return 0;
    }

    struct fi_rma_iov rma_iov;
    rma_iov.addr = (uint64_t)mr->start;
    rma_iov.key = fi_mr_key(mr->fid);
    DEBUG_MSG("Exchanging MR Info (key: 0x%lx, addr: 0x%lx)....", rma_iov.key, rma_iov.addr);

    int ret = rt_exchange_data("mr_info", &rma_iov, sizeof(struct fi_rma_iov), mr->iov, rofi->desc.nid, rofi->desc.nodes);
    if (ret) {
        ERR_MSG("Error exchanging info for memory region alloc buffer. Aborting!");
        return ret;
    }

#ifdef _DEBUG
    for (int i = 0; i < rofi->desc.nodes; i++) {
        DEBUG_MSG("\t Node: %d Key: 0x%lx Addr: 0x%lx", i, mr->iov[i].key, mr->iov[i].addr);
    }
#endif
    return 0;
}