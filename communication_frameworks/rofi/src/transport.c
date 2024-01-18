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
#include <math.h>
#include <netdb.h>
#include <poll.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#include <rdma/fabric.h>
#include <rdma/fi_cm.h>
#include <rdma/fi_collective.h>
#include <rdma/fi_domain.h>
#include <rdma/fi_endpoint.h>
#include <rdma/fi_errno.h>
#include <rdma/fi_rma.h>

#include "rofi_debug.h"
#include "rofi_internal.h"
#include "transport.h"

int rofi_transport_fini(rofi_transport_t *rofi) {
    DEBUG_MSG("Fini");

    int ret = fi_close(&rofi->ep->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        rofi->ep = NULL;
    }
    DEBUG_MSG("ep closed");

    ret = fi_close(&rofi->cq->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fl_close", ret);
        rofi->cq = NULL;
    }
    DEBUG_MSG("cq closed");

    ret = fi_close(&rofi->put_cntr->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        rofi->put_cntr = NULL;
    }
    DEBUG_MSG("put_cntr closed");

    ret = fi_close(&rofi->get_cntr->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        rofi->get_cntr = NULL;
    }
    DEBUG_MSG("get_cntr closed");

    ret = fi_close(&rofi->av->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        rofi->av = NULL;
    }
    DEBUG_MSG("av closed");

    ret = fi_close(&rofi->eq->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        rofi->eq = NULL;
    }
    DEBUG_MSG("eq closed");

    ret = fi_close(&rofi->domain->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        rofi->domain = NULL;
    }
    DEBUG_MSG("domain closed");

    ret = fi_close(&rofi->fabric->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        rofi->fabric = NULL;
    }
    DEBUG_MSG("fabric closed");
    free(rofi->info);
    DEBUG_MSG("info freed");
    return 0;
}

int rofi_transport_init(struct fi_info *hints, rofi_transport_t *rofi) {
    DEBUG_MSG("fi_getinfo");
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

    struct fi_info *prov = rofi->info;
    while (prov != NULL) {
        DEBUG_MSG("\tSelected Provider: %s  Version: (%u.%u) Fabric: %s Domain: %s max_inject: %zu, max_msg: %zu, stx: %s, MR_RMA_EVENT: %s, msg: %s, rma: %s, read: %s, write: %s, remote_read: %s, remote_write: %s, rma_event: %s, atomic: %s, collective: %s, src_addr %s, src_addrlen %lu, dest_addr %s, dest_addrlen %lu",
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
                  rofi->info->caps & FI_ATOMIC ? "yes" : "no",
                  rofi->info->caps & FI_COLLECTIVE ? "yes" : "no",
                  rofi->info->src_addr, rofi->info->src_addrlen,
                  rofi->info->dest_addr, rofi->info->dest_addrlen);
        prov = prov->next;
    }

    rofi->desc.max_message_size = rofi->info->ep_attr->max_msg_size;
    rofi->desc.inject_size = rofi->info->tx_attr->inject_size;

    struct fi_collective_attr attr = {0};
    attr.op = FI_ATOMIC_READ;
    attr.datatype = FI_UINT64;
    attr.mode = 0;
    DEBUG_MSG("fi_query_collective: FI_ALLGATHER");
    ret = fi_query_collective(rofi->domain, FI_ALLGATHER, &attr, 0);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_query_collective", ret);
        return (ret);
    }
    else {
        DEBUG_MSG("fi_query_collective: FI_ALLGATHER supported");
    }

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
    DEBUG_MSG("FI_FABRIC");
    int ret = fi_fabric(rofi->info->fabric_attr, &rofi->fabric, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_fabric", ret);
        return ret;
    }

    struct fi_eq_attr eq_attr = {0};
    eq_attr.wait_obj = FI_WAIT_UNSPEC;
    DEBUG_MSG("FI_EQ_OPEN");
    ret = fi_eq_open(rofi->fabric, &eq_attr, &rofi->eq, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_eq_open", ret);
        return ret;
    }

    DEBUG_MSG("FI_DOMAIN");
    ret = fi_domain(rofi->fabric, rofi->info, &rofi->domain, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_domain", ret);
        return ret;
    }

    return 0;
}
int rofi_transport_init_endpoint_resources(rofi_transport_t *rofi) {
    struct fi_cntr_attr put_cntr_attr = {0};
    struct fi_cntr_attr get_cntr_attr = {0};
    put_cntr_attr.events = FI_CNTR_EVENTS_COMP;
    get_cntr_attr.events = FI_CNTR_EVENTS_COMP;
    put_cntr_attr.wait_obj = FI_WAIT_UNSPEC;
    get_cntr_attr.wait_obj = FI_WAIT_UNSPEC;

    DEBUG_MSG("put FI_CNTR_OPEN");
    int ret = fi_cntr_open(rofi->domain, &put_cntr_attr, &rofi->put_cntr, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_cntr_open", ret);
        return ret;
    }

    DEBUG_MSG("get FI_CNTR_OPEN");
    ret = fi_cntr_open(rofi->domain, &get_cntr_attr, &rofi->get_cntr, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_cntr_open", ret);
        return ret;
    }

    struct fi_cq_attr cq_attr = {0};
    cq_attr.format = FI_CQ_FORMAT_CONTEXT;

    DEBUG_MSG("FI_CQ_OPEN");
    ret = fi_cq_open(rofi->domain, &cq_attr, &rofi->cq, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_cq_open", ret);
        return ret;
    }

    struct fi_av_attr av_attr = {0};
    if (rofi->info->domain_attr->av_type != FI_AV_UNSPEC) {
        av_attr.type = rofi->info->domain_attr->av_type;
    }

    DEBUG_MSG("FI_AV_OPEN");
    ret = fi_av_open(rofi->domain, &av_attr, &rofi->av, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_av_open", ret);
        return ret;
    }

    rofi->info->ep_attr->tx_ctx_cnt = 0;
    rofi->info->caps = FI_RMA | FI_WRITE | FI_READ | FI_COLLECTIVE;
    rofi->info->tx_attr->op_flags = FI_DELIVERY_COMPLETE; // FI_TRANSMIT_COMPLETE fails, FI_DELIVERY_COMPLETE works but I dont see a difference?
    rofi->info->mode = 0;
    rofi->info->tx_attr->mode = 0;
    rofi->info->rx_attr->mode = 0;
    rofi->info->tx_attr->caps = rofi->info->caps;
    rofi->info->rx_attr->caps = FI_RECV | FI_COLLECTIVE; // to drive progress

    DEBUG_MSG("FI_ENDPOINT");
    ret = fi_endpoint(rofi->domain, rofi->info, &rofi->ep, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_endpoint", ret);
        return ret;
    }

    // bind event queue
    DEBUG_MSG("FI_EP_BIND eq");
    ret = fi_ep_bind(rofi->ep, &rofi->eq->fid, 0);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_ep_bind eq", ret);
        return ret;
    }

    // bind address vector
    DEBUG_MSG("FI_EP_BIND av");
    ret = fi_ep_bind(rofi->ep, &rofi->av->fid, 0);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_ep_bind av", ret);
        return ret;
    }

    // bind put cntr
    DEBUG_MSG("FI_EP_BIND put_cntr");
    ret = fi_ep_bind(rofi->ep, &rofi->put_cntr->fid, FI_WRITE | FI_REMOTE_WRITE);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_ep_bind put_cntr", ret);
        return ret;
    }

    // bind get cntr
    DEBUG_MSG("FI_EP_BIND get_cntr");
    ret = fi_ep_bind(rofi->ep, &rofi->get_cntr->fid, FI_READ);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_ep_bind get_cntr", ret);
        return ret;
    }

    // bind cq -- use same completion queue for send and recv
    DEBUG_MSG("FI_EP_BIND cq");
    ret = fi_ep_bind(rofi->ep, &rofi->cq->fid, FI_SELECTIVE_COMPLETION | FI_TRANSMIT | FI_RECV);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_ep_bind cq", ret);
        return ret;
    }

    DEBUG_MSG("FI_ENABLE");
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
            return ret;
        }
    }

    DEBUG_MSG("FI_AV_INSERT");
    int ret = fi_av_insert(rofi->av, all_addrs, rofi->desc.nodes, rofi->remote_addrs, 0, NULL);
    if (ret < 0) {
        ROFI_TRANSPORT_ERR_MSG("ft_av_insert", ret);
        return ret;
    }
    else if (ret != rofi->desc.nodes) {
        ERR_MSG("fi_av_insert: number of addresses inserted = %d;"
                " number of addresses given = %d\n",
                ret, rofi->desc.nodes);
        return ret;
    }
    return 0;
}

// only need this if we use MANUAL_PROGRESS
// because we are using FI_SELECTIVE_COMPLETION
// successfull completions should increment the respective cntrs
// this shouldn't return any completions, as thc cq should
// only handle error events now
int rofi_transport_progress(rofi_transport_t *rofi) {
    struct fi_cq_entry buf = {0};
    int ret = fi_cq_read(rofi->cq, &buf, 1);
    if (ret == 1) {
        printf("unexpected cq event\n"); // warnd
    }
    else if (ret < 0 && ret != -FI_EAGAIN) {
        ROFI_TRANSPORT_ERR_MSG("fi_cq_read", ret);
        struct fi_cq_err_entry ebuf = {0};
        int ret = fi_cq_readerr(rofi->cq, (void *)&ebuf, 0);
        if (ret > 0) {
            const char *errmsg = fi_cq_strerror(rofi->cq, ebuf.prov_errno, ebuf.err_data, NULL, 0);
            ERR_MSG("Error: %s\n", errmsg);
            abort();
            return ret;
        }
        else if (ret && ret != -FI_EAGAIN) {
            ROFI_TRANSPORT_ERR_MSG("fi_cq_readerr", ret);
            return ret;
        }
        return (ret);
    }
    return 0;
}

// checks the rma return status, and aborts if not -FI_EAGAIN
// note this does not try to make progress
int rofi_transport_ctx_check_err(rofi_transport_t *rofi, int err) {
    if (err == -FI_EAGAIN) {
        struct fi_cq_err_entry ebuf = {0};
        pthread_mutex_lock(&rofi->lock);
        int ret = fi_cq_readerr(rofi->cq, (void *)&ebuf, 0);
        pthread_mutex_unlock(&rofi->lock);
        if (ret > 0) {
            const char *errmsg = fi_cq_strerror(rofi->cq, ebuf.prov_errno, ebuf.err_data, NULL, 0);
            ERR_MSG("Error: %s\n", errmsg);
            return ret;
        }
        else if (ret && ret != -FI_EAGAIN) {
            ROFI_TRANSPORT_ERR_MSG("fi_cq_readerr", ret);
            return ret;
        }
    }
    else if (err) {
        ROFI_TRANSPORT_ERR_MSG("", err);
        return err;
    }
    return 0;
}

// checks the rma return status, and aborts if not -FI_EAGAIN
// otherwise tries to make progress
int rofi_transport_check_rma_err(rofi_transport_t *rofi, int err) {
    if (err == -FI_EAGAIN) {
        struct fi_cq_err_entry ebuf = {0};
        int ret = fi_cq_readerr(rofi->cq, (void *)&ebuf, 0);
        if (ret > 0) {
            const char *errmsg = fi_cq_strerror(rofi->cq, ebuf.prov_errno, ebuf.err_data, NULL, 0);
            printf("Error: %s\n", errmsg);
            return ret;
        }
        else if (ret && ret != -FI_EAGAIN) {
            ROFI_TRANSPORT_ERR_MSG("fi_cq_readerr", ret);
            return ret;
        }
        ret = rofi_transport_progress(rofi);
        if (ret) {
            return ret;
        }
    }
    else if (err) {
        ROFI_TRANSPORT_ERR_MSG("", err);
        return err;
    }
    return 0;
}

int rofi_transport_wait_on_cntr(rofi_transport_t *rofi, uint64_t *pending_cntr, struct fid_cntr *cntr) {
    uint64_t cnt = *pending_cntr;
    uint64_t prev_cnt = cnt;
    pthread_mutex_lock(&rofi->lock);
    uint64_t cur_cnt = fi_cntr_read(cntr);
    uint64_t err_cnt = fi_cntr_readerr(cntr);
    pthread_mutex_unlock(&rofi->lock);
    DEBUG_MSG("Waiting for  %lu  cnts... cur_cnt: %lu err_cnt: %lu", cnt, cur_cnt, err_cnt);
    do {
        prev_cnt = cnt;
        pthread_mutex_lock(&rofi->lock);
        int ret = fi_cntr_wait(cntr, prev_cnt, -1);
        pthread_mutex_unlock(&rofi->lock);
        cnt = *pending_cntr; // this could be updated by another thread
        ret = rofi_transport_ctx_check_err(rofi, ret);
        if (ret) {
            return ret;
        }
    } while (prev_cnt < cnt);
    // pthread_mutex_lock(&rofi->lock);
    // uint64_t cnt = fi_cntr_read(cntr);
    // uint64_t err_cnt = fi_cntr_readerr(cntr);
    // pthread_mutex_unlock(&rofi->lock);
    // DEBUG_MSG("Done Waiting for  %lu  prev_cnt: %lu gets to complete... cnt: %lu err_cnt: %lu", *pending_cntr, prev_cnt, cnt, err_cnt);
    assert(prev_cnt == cnt);
    return 0;
}

int rofi_transport_put_inject(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, fi_addr_t pe, const void *src_addr, size_t len) {
    pthread_mutex_lock(&rofi->lock);
    rofi->pending_put_cntr += 1;
    DEBUG_MSG("fi_inject_write %p %p %d %d %p 0x%lx", rofi->ep, src_addr, len, pe, rma_iov->addr, rma_iov->key);
    int ret = fi_inject_write(rofi->ep, src_addr, len, pe, rma_iov->addr, rma_iov->key);
    while (ret) { // retry while FI_EAGAIN
        ret = rofi_transport_check_rma_err(rofi, ret);
        if (ret) {
            return ret;
        }
        ret = fi_inject_write(rofi->ep, src_addr, len, pe, rma_iov->addr, rma_iov->key);
    }
    pthread_mutex_unlock(&rofi->lock);
    DEBUG_MSG("fi_inject_write done %p %p %d %d %p 0x%lx", rofi->ep, src_addr, len, pe, rma_iov->addr, rma_iov->key);
    return 0;
}

int rofi_transport_put_large(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, fi_addr_t pe, const void *src_addr, size_t len, void *desc, void *context) {

    uint8_t *src_cur_addr = (uint8_t *)src_addr;
    uint8_t *src_end_addr = src_cur_addr + len;
    uint64_t dst_cur_addr = (uint64_t)rma_iov->addr;
    pthread_mutex_lock(&rofi->lock);
    DEBUG_MSG("fi_write %p %p %d %d %p 0x%lx", rofi->ep, src_cur_addr, len, pe, rma_iov->addr, rma_iov->key);
    while (src_cur_addr < src_end_addr) {
        uint64_t cur_len = MIN(src_end_addr - src_cur_addr, rofi->desc.max_message_size);
        rofi->pending_put_cntr += 1;

        int ret = fi_write(rofi->ep, src_cur_addr, cur_len, desc, pe, dst_cur_addr, rma_iov->key, context);

        while (ret) { // retry while FI_EAGAIN
            ret = rofi_transport_check_rma_err(rofi, ret);
            if (ret) {
                return ret;
            }
            ret = fi_write(rofi->ep, src_cur_addr, cur_len, desc, pe, dst_cur_addr, rma_iov->key, context);
        }
        src_cur_addr += cur_len;
        dst_cur_addr += cur_len;
    }
    pthread_mutex_unlock(&rofi->lock);
    DEBUG_MSG("fi_inject_write %p %p %d %d %p 0x%lx", rofi->ep, src_addr, len, pe, rma_iov->addr, rma_iov->key);
    return 0;
}

// for PE need to check if using FI_AV_MAP, and then index into that
int rofi_transport_put(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, const void *src_addr, size_t len, void *desc, void *context) {
    if (len < rofi->desc.inject_size) {
        return rofi_transport_put_inject(rofi, rma_iov, rofi->remote_addrs[pe], src_addr, len);
    }
    else {
        return rofi_transport_put_large(rofi, rma_iov, rofi->remote_addrs[pe], src_addr, len, desc, context);
    }
}

int rofi_transport_put_wait_all(rofi_transport_t *rofi) {
    return rofi_transport_wait_on_cntr(rofi, &rofi->pending_put_cntr, rofi->put_cntr);
}

int rofi_transport_get_small(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, void *dst_addr, size_t len, void *desc, void *context) {

    pthread_mutex_lock(&rofi->lock);
    rofi->pending_get_cntr += 1;
    int ret = fi_read(rofi->ep, dst_addr, len, desc, pe, rma_iov->addr, rma_iov->key, context);
    while (ret) { // retry while FI_EAGAIN
        ret = rofi_transport_check_rma_err(rofi, ret);
        if (ret) {
            return ret;
        }
        ret = fi_read(rofi->ep, dst_addr, len, desc, pe, rma_iov->addr, rma_iov->key, context);
    }
    pthread_mutex_unlock(&rofi->lock);
    return 0;
}

int rofi_transport_get_large(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, void *dst_addr, size_t len, void *desc, void *context) {
    uint64_t src_cur_addr = (uint64_t)rma_iov->addr;

    uint8_t *dst_cur_addr = (uint8_t *)dst_addr;
    uint8_t *dst_end_addr = dst_cur_addr + len;
    pthread_mutex_lock(&rofi->lock);
    while (dst_cur_addr < dst_end_addr) {
        uint64_t cur_len = MIN(dst_end_addr - dst_cur_addr, rofi->desc.max_message_size);
        rofi->pending_get_cntr += 1;

        int ret = fi_read(rofi->ep, dst_cur_addr, cur_len, desc, pe, src_cur_addr, rma_iov->key, context);

        while (ret) { // retry while FI_EAGAIN
            ret = rofi_transport_check_rma_err(rofi, ret);
            if (ret) {
                return ret;
            }
            ret = fi_read(rofi->ep, dst_cur_addr, cur_len, desc, pe, src_cur_addr, rma_iov->key, context);
        }
        src_cur_addr += cur_len;
        dst_cur_addr += cur_len;
    }
    pthread_mutex_unlock(&rofi->lock);
    return 0;
}

int rofi_transport_get(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, void *dst_addr, size_t len, void *desc, void *context) {
    if (len < rofi->desc.max_message_size) {
        return rofi_transport_get_small(rofi, rma_iov, rofi->remote_addrs[pe], dst_addr, len, desc, context);
    }
    else {
        return rofi_transport_get_large(rofi, rma_iov, rofi->remote_addrs[pe], dst_addr, len, desc, context);
    }
}

int rofi_transport_get_wait_all(rofi_transport_t *rofi) {
    return rofi_transport_wait_on_cntr(rofi, &rofi->pending_get_cntr, rofi->get_cntr);
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

int rofi_transport_wait_on_event(rofi_transport_t *rofi, uint32_t event, void *context) {
    uint32_t ev;
    struct fi_eq_entry entry;

    while (true) {
        int ret = fi_eq_read(rofi->eq, &ev, &entry, sizeof(entry), 0);
        if (ret >= 0) { // we got an event
            if (ev == event) {
                if (!context || (context == entry.context)) {
                    return 0;
                }
                else if (context) {
                    return -FI_EOTHER;
                }
            }
        }
        else if (ret != -FI_EAGAIN) {
            ROFI_TRANSPORT_ERR_MSG("fi_eq_read", ret);
            return ret;
        }
        ret = rofi_transport_progress(rofi);
        if (ret) {
            return ret;
        }
    }
}

int rofi_transport_wait_on_context_comp(rofi_transport_t *rofi, void *context) {
    struct fi_cq_entry buf = {0};
    struct fi_cq_err_entry err_entry = {0};

    while (true) {
        int ret = fi_cq_read(rofi->cq, &buf, 1);
        if (ret < 0 && ret != -FI_EAGAIN) {
            ROFI_TRANSPORT_ERR_MSG("fi_cq_read", ret);
            return ret;
        }
        if (buf.op_context && buf.op_context == context) {
            return 0;
        }
    }
}

// TODO Can we replace this with libfabrics broadcast?
int rofi_transport_sub_exchange_mr_info(rofi_transport_t *rofi, rofi_mr_desc *mr, uint64_t *pes, uint64_t num_pes) {
    if (rofi->desc.nodes == 1) {
        return 0;
    }

    int me = rofi->desc.nid;
    if (pes != NULL) { // doing sub barrier, figure out team pe id
        for (int i = 0; i < num_pes; i++) {
            if (pes[i] == me) {
                me = i;
                break;
            }
        }
    }

    DEBUG_MSG("Broadcasting MR Info (key: 0x%lx, 0x%lx, addr: 0x%lx) to %d PEs....", mr->mr_key, fi_mr_key(mr->fid), mr->start, num_pes);
    struct fi_av_set_attr av_set_attr = {0};
    av_set_attr.count = num_pes;
    av_set_attr.start_addr = rofi->remote_addrs[pes[0]];
    av_set_attr.end_addr = rofi->remote_addrs[pes[0]];
    av_set_attr.stride = 1;
    // av_set_attr.comm_key_size = 0; // need to look into comm keys more
    // av_set_attr.comm_key = 0;
    av_set_attr.flags = 0;

    struct fid_av_set *av_set;
    pthread_mutex_lock(&rofi->lock);
    int ret = fi_av_set(rofi->av, &av_set_attr, &av_set, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_av_st", ret);
        pthread_mutex_unlock(&rofi->lock);
        return ret;
    }
    DEBUG_MSG("CREATED AV_SET: 0x%p", av_set);

    for (int i = 1; i < num_pes; i++) {
        ret = fi_av_set_insert(av_set, rofi->remote_addrs[pes[i]]);
        if (ret) {
            ROFI_TRANSPORT_ERR_MSG("fi_av_set_insert", ret);
            pthread_mutex_unlock(&rofi->lock);
            return ret;
        }
        DEBUG_MSG("Inserted PE %d into AV_SET: 0x%p", pes[i], av_set);
    }
    fi_addr_t coll_addr = 0;
    ret = fi_av_set_addr(av_set, &coll_addr);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_av_set_addr", ret);
        pthread_mutex_unlock(&rofi->lock);
        return ret;
    }

    DEBUG_MSG("COLL_ADDR: 0x%p", coll_addr);

    uint64_t done_flag;
    struct fid_mc *mc;
    ret = fi_join_collective(rofi->ep, coll_addr, av_set, 0, &mc, &done_flag);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_join_collective", ret);
        pthread_mutex_unlock(&rofi->lock);
        return ret;
    }
    DEBUG_MSG("Initiated collective join...");
    ret = rofi_transport_wait_on_event(rofi, FI_JOIN_COMPLETE, &done_flag);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("rofi_transport_wait_on_event", ret);
        pthread_mutex_unlock(&rofi->lock);
        return ret;
    }
    DEBUG_MSG("Joined collective");

    struct fi_rma_iov rma_iov;
    rma_iov.addr = (uint64_t)mr->start;
    rma_iov.key = fi_mr_key(mr->fid);

    struct fi_rma_iov *results = malloc(num_pes * sizeof(struct fi_rma_iov));
    if (results == NULL) {
        pthread_mutex_unlock(&rofi->lock);
        ERR_MSG("malloc failed");
        return -1;
    }

    fi_addr_t coll_addr2 = fi_mc_addr(mc);
    ret = fi_allgather(rofi->ep, &rma_iov, sizeof(rma_iov), NULL, results, NULL, coll_addr2, FI_UINT8, 0, &done_flag);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_allgather", ret);
        pthread_mutex_unlock(&rofi->lock);
        free(results);
        return ret;
    }
    ret = rofi_transport_wait_on_context_comp(rofi, &done_flag);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("rofi_transport_wait_on_event", ret);
        pthread_mutex_unlock(&rofi->lock);
        free(results);
        return ret;
    }

    for (int i = 0; i < num_pes; i++) {
        mr->iov[pes[i]] = results[i];
        DEBUG_MSG("i: %d(pe: %d), addr: 0x%lx, key: 0x%lx  ", i, pes[i], results[i].addr, results[i].key);
    }
    free(results);
    // for now we will immediately close down the collective group as we are currently only using them to do the addr+key transfer
    // in the future we probably want the collective group to persist as long as the memory region?
    DEBUG_MSG("Closing collective group");
    ret = fi_close(&mc->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        DEBUG_MSG("CLOSING AV SET");
        fi_close(&av_set->fid);
        if (ret) {
            ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        }
    }
    else {
        DEBUG_MSG("CLOSING AV SET");
        ret = fi_close(&av_set->fid);
        if (ret) {
            ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        }
    }

    pthread_mutex_unlock(&rofi->lock);
    return ret;

    //     int me = rofi->desc.nid;

    //     if (pes != NULL) { // doing sub barrier, figure out team pe id
    //         for (int i = 0; i < num_pes; i++) {
    //             if (pes[i] == me) {
    //                 me = i;
    //                 break;
    //             }
    //         }
    //     }

    //     struct fi_rma_iov rma_iov;
    //     rma_iov.addr = (uint64_t)mr->start;
    //     rma_iov.key = fi_mr_key(mr->fid);

    //     rofi->alloc_bufs.iov_buf[rofi->desc.nid] = rma_iov; // we are only allowed to write into our own buffer slot
    //     int ret = rofi_transport_sub_barrier(rofi, pes, me, num_pes);
    //     if (ret) {
    //         return ret;
    //     }
    //     // DEBUG_MSG("putting  Node: (%d) Key: 0x%lx Addr: 0x%lx", rofi->desc.nid, rma_iov.key, rma_iov.addr);
    //     bool ready = false;
    //     uint64_t prev_addr_hash = 0;
    //     uint64_t prev_key_hash = 0;
    //     uint64_t cnt = 0;

    //     while (!ready) {
    //         // start at next node to the "right" of me to try to avoid all nodes sending to the same nodes at the same time
    //         for (int i = me + 1; i < num_pes; i++) {
    //             rofi->alloc_bufs.iov_buf[i] = rofi->alloc_bufs.iov_buf[rofi->desc.nid];
    //             ret = rofi_get_internal(&rofi->alloc_bufs.iov_buf[pes[i]], &rofi->alloc_bufs.iov_buf[pes[i]], sizeof(struct fi_rma_iov), pes[i], ROFI_ASYNC);
    //             if (ret) {
    //                 return ret;
    //             }
    //         }
    //         for (int i = 0; i < me; i++) {
    //             rofi->alloc_bufs.iov_buf[i] = rofi->alloc_bufs.iov_buf[rofi->desc.nid];
    //             ret = rofi_get_internal(&rofi->alloc_bufs.iov_buf[pes[i]], &rofi->alloc_bufs.iov_buf[pes[i]], sizeof(struct fi_rma_iov), pes[i], ROFI_ASYNC);
    //             if (ret) {
    //                 return ret;
    //             }
    //         }
    //         ret = rofi_wait_internal();
    //         if (ret) {
    //             return ret;
    //         }
    //         ret = rofi_transport_sub_barrier(rofi, pes, me, num_pes);
    //         if (ret) {
    //             return ret;
    //         }

    //         // calculate a simple hash, and then get the hash from every PE to ensure we received the same value

    //         // DEBUG_MSG("in ready loop");
    //         rofi->alloc_bufs.hash_buf[rofi->desc.nid * 2] = 0;
    //         rofi->alloc_bufs.hash_buf[rofi->desc.nid * 2 + 1] = 0;
    //         for (int i = 0; i < num_pes; i++) {
    //             rofi->alloc_bufs.hash_buf[rofi->desc.nid * 2] += rofi->alloc_bufs.iov_buf[pes[i]].addr;
    //             rofi->alloc_bufs.hash_buf[rofi->desc.nid * 2 + 1] += rofi->alloc_bufs.iov_buf[pes[i]].key;
    //         }
    //         ret = rofi_transport_sub_barrier(rofi, pes, me, num_pes);
    //         if (ret) {
    //             return ret;
    //         }
    //         // DEBUG_MSG("get hashes {0x%lx, 0x%lx}", rofi->alloc_bufs.hash_buf[rofi->desc.nid * 2], rofi->alloc_bufs.hash_buf[rofi->desc.nid * 2 + 1]);
    //         for (int i = me + 1; i < num_pes; i++) {
    //             ret = rofi_get_internal(&rofi->alloc_bufs.hash_buf[pes[i] * 2], &rofi->alloc_bufs.hash_buf[pes[i] * 2], 2 * sizeof(uint64_t), pes[i], ROFI_ASYNC);
    //             if (ret) {
    //                 return ret;
    //             }
    //             cnt = 0;
    //         }
    //         for (int i = 0; i < me; i++) {
    //             ret = rofi_get_internal(&rofi->alloc_bufs.hash_buf[pes[i] * 2], &rofi->alloc_bufs.hash_buf[pes[i] * 2], 2 * sizeof(uint64_t), pes[i], ROFI_ASYNC);
    //             if (ret) {
    //                 return ret;
    //             }
    //         }
    //         ret = rofi_wait_internal();
    //         if (ret) {
    //             return ret;
    //         }

    //         //         DEBUG_MSG("Hash check: prev addr hash 0x%lx, curr addr hash 0x%lx, prev key hash 0x%lx, curr key hash 0x%lx", prev_addr_hash, rofi->alloc_bufs.hash_buf[rofi->desc.nid * 2], prev_key_hash, rofi->alloc_bufs.hash_buf[rofi->desc.nid * 2 + 1]);
    //         // #ifdef _DEBUG
    //         //         for (int i = 0; i < num_pes; i++) {
    //         //             printf("Hash check: i: %d(pe: %d), addr: 0x%lx, key: 0x%lx  ", i, pes[i], rofi->alloc_bufs.hash_buf[pes[i] * 2], rofi->alloc_bufs.hash_buf[pes[i] * 2 + 1]);
    //         //         }
    //         //         printf("\n");
    //         //         for (int i = 0; i < num_pes; i++) {
    //         //             printf("Hash check: i: %d(pe: %x), addr: 0x%lx, key: 0x%lx  ", i, pes[i], rofi->alloc_bufs.iov_buf[pes[i]].addr, rofi->alloc_bufs.iov_buf[pes[i]].key);
    //         //         }
    //         //         printf("\n");
    //         // #endif
    //         ready = true;
    //         ret = rofi_transport_sub_barrier(rofi, pes, me, num_pes);
    //         if (ret) {
    //             return ret;
    //         }
    //         for (int i = 0; i < num_pes; i++) {
    //             ready &= rofi->alloc_bufs.hash_buf[rofi->desc.nid * 2] == rofi->alloc_bufs.hash_buf[pes[i] * 2];
    //             ready &= rofi->alloc_bufs.hash_buf[rofi->desc.nid * 2 + 1] == rofi->alloc_bufs.hash_buf[pes[i] * 2 + 1];
    //         }
    // // if (cnt % 1000 == 0) {
    // #ifdef _DEBUG
    //         DEBUG_MSG("prev addr hash 0x%lx, curr addr hash 0x%lx, prev key hash 0x%lx, curr key hash 0x%lx", prev_addr_hash, rofi->alloc_bufs.hash_buf[rofi->desc.nid * 2], prev_key_hash, rofi->alloc_bufs.hash_buf[rofi->desc.nid * 2 + 1]);
    //         for (int i = 0; i < num_pes; i++) {
    //             printf("i: %d(pe: %d), addr: 0x%lx, key: 0x%lx  ", i, pes[i], rofi->alloc_bufs.hash_buf[pes[i] * 2], rofi->alloc_bufs.hash_buf[pes[i] * 2 + 1]);
    //         }
    //         printf("\n");
    //         for (int i = 0; i < num_pes; i++) {
    //             printf("i: %d(pe: %x), addr: 0x%lx, key: 0x%lx  ", i, pes[i], rofi->alloc_bufs.iov_buf[pes[i]].addr, rofi->alloc_bufs.iov_buf[pes[i]].key);
    //         }
    //         printf("\n");
    // #endif
    //         // }
    //         if (prev_addr_hash == rofi->alloc_bufs.hash_buf[rofi->desc.nid * 2] && prev_key_hash == rofi->alloc_bufs.hash_buf[rofi->desc.nid * 2 + 1]) {
    //             cnt += 1;
    //         }
    //         prev_addr_hash = rofi->alloc_bufs.hash_buf[rofi->desc.nid * 2];
    //         prev_key_hash = rofi->alloc_bufs.hash_buf[rofi->desc.nid * 2 + 1];
    //         // cnt += 1;
    //         ret = rofi_transport_sub_barrier(rofi, pes, me, num_pes);
    //         if (ret) {
    //             return ret;
    //         }
    //         if (cnt > 1000000) {
    //             ERR_MSG("EXCHANGE failed, cnt: %d", cnt);
    //             return -1;
    //         }
    //     }
    //     for (int i = 0; i < num_pes; i++) {
    //         mr->iov[pes[i]] = rofi->alloc_bufs.iov_buf[pes[i]];
    //         DEBUG_MSG("i: %d(pe: %d), addr: 0x%lx, key: 0x%lx  ", i, pes[i], rofi->alloc_bufs.iov_buf[pes[i]].addr, rofi->alloc_bufs.iov_buf[pes[i]].key);
    //     }
    //     for (int i = 0; i < rofi->desc.nid; i++) {
    //         rofi->alloc_bufs.iov_buf[i].key = 0;
    //         rofi->alloc_bufs.iov_buf[i].addr = 0;
    //         rofi->alloc_bufs.hash_buf[i * 2] = 0;
    //         rofi->alloc_bufs.hash_buf[i * 2 + 1] = 0;
    //         rofi->alloc_bufs.barrier_buf[i] = 0;
    //     }
    //     ret = rofi_transport_sub_barrier_reset(rofi, pes, me, num_pes);
    //     for (int i = 0; i < rofi->desc.nid; i++) {
    //         rofi->alloc_bufs.reset_barrier_buf[i] = 0;
    //     }
    //     rofi->alloc_bufs.barrier_id = 0;
    //     if (ret) {
    //         return ret;
    //     }
    //     return 0;
}

int euclid_rem(int a, int b) {
    int r = a % b;
    return r >= 0 ? r : r + abs(b);
}

int rofi_transport_inner_barrier(rofi_transport_t *rofi, uint64_t *barrier_id, uint64_t *barrier_buf, uint64_t *pes, uint64_t me, uint64_t num_pes) {
    int n = 2;

    int num_rounds = ceil(log2((double)num_pes) / log2((double)n));

    int ret = 0;

    *barrier_id += 1;
    void *src = (void *)barrier_id;
    for (int round = 0; round < num_rounds; round++) {
        for (int i = 1; i <= n; i++) {
            int send_pe = euclid_rem((int)(me + i * pow(n + 1, round)), num_pes);
            send_pe = pes == NULL ? send_pe : pes[send_pe]; // if pes not null we are doing sub barrier

            // we need to store in the absolute pe location to prevent races.
            // allocations on multiple teams including the same PE can occur simultaneously,
            // the upper level runtime must ensure a given PE is only participating in one allocation at a time
            void *dst = (void *)(&barrier_buf[rofi->desc.nid]);

            struct fi_rma_iov rma_iov;
            rma_iov.addr = (uint64_t)(dst - rofi->mr->start + rofi->mr->iov[send_pe].addr);
            rma_iov.key = rofi->mr->iov[send_pe].key;
            DEBUG_MSG("%d Sending %d to %d %p", me, *barrier_id, send_pe, dst);
            ret = rofi_transport_put(rofi, &rma_iov, send_pe, src, sizeof(uint64_t), rofi->mr->mr_desc, NULL);
            if (ret) {
                return ret;
            }
        }
        for (int i = 1; i <= n; i++) {
            int recv_pe = euclid_rem((int)(me - i * pow(n + 1, round)), num_pes);
            recv_pe = pes == NULL ? recv_pe : pes[recv_pe]; // if pes not null we are doing sub barrier
            DEBUG_MSG("%d Receiving %d from %d", me, *barrier_id, recv_pe);

            while (barrier_buf[recv_pe] < *barrier_id) {
                pthread_mutex_lock(&rofi->lock);
                ret = rofi_transport_progress(rofi);
                if (ret) {
                    return ret;
                }
                pthread_mutex_unlock(&rofi->lock);
                sched_yield();
            }
        }
    }
    return 0;
}

int rofi_transport_barrier(rofi_transport_t *rofi) {
    return rofi_transport_inner_barrier(rofi, &rofi->global_barrier_id, rofi->global_barrier_buf, NULL, rofi->desc.nid, rofi->desc.nodes);
}

int rofi_transport_sub_barrier(rofi_transport_t *rofi, uint64_t *pes, uint64_t me, uint64_t num_pes) {
    return rofi_transport_inner_barrier(rofi, &rofi->alloc_bufs.barrier_id, rofi->alloc_bufs.barrier_buf, pes, me, num_pes);
}
int rofi_transport_sub_barrier_reset(rofi_transport_t *rofi, uint64_t *pes, uint64_t me, uint64_t num_pes) {
    return rofi_transport_inner_barrier(rofi, &rofi->alloc_bufs.barrier_id, rofi->alloc_bufs.reset_barrier_buf, pes, me, num_pes);
}