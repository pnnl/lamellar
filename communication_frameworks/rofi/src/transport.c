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

static inline void atomic_max_u64(_Atomic uint64_t *p, uint64_t v) {
    uint64_t old = atomic_load_explicit(p, memory_order_relaxed);
    while (old < v) {
        if (atomic_compare_exchange_weak_explicit(p, &old, v, memory_order_relaxed, memory_order_relaxed)) {
            break;
        }
        // old updated by compare_exchange
    }
}


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

    ret = fi_close(&rofi->send_cntr->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        rofi->put_cntr = NULL;
    }
    DEBUG_MSG("send_cntr closed");

    ret = fi_close(&rofi->recv_cntr->fid);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_close", ret);
        rofi->get_cntr = NULL;
    }
    DEBUG_MSG("recv_cntr closed");

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
    fi_freeinfo(rofi->info);
    DEBUG_MSG("info freed");
    return 0;
}

void rofi_transport_select_provider(struct fi_info *prov, rofi_transport_t *rofi, rofi_names_t *prov_names, rofi_names_t *domain_names) {
    DEBUG_MSG("Selecting Provider: %p %p", prov_names, domain_names);
    struct fi_info *prov_cur = prov;
    struct fi_info *prov_found = NULL;
    if (prov_names == NULL && domain_names == NULL) {
        rofi->info = fi_dupinfo(prov_cur);
        WARN_MSG("No matches for the specified provider and/or domain: NULL NULL");
        WARN_MSG("Using first available provider: %s %s", prov_cur->fabric_attr->prov_name, prov_cur->domain_attr->name);
        
        return;
    }
    else {
        while (prov_cur != NULL) {
            if (prov_names != NULL) {
                for (int i = 0; i < prov_names->num; i++) {
                    DEBUG_MSG("checking Provider (%s): %s %s", prov_names->names[i], prov_cur->fabric_attr->prov_name, prov_cur->domain_attr->name);
                    if (strncmp(prov_cur->fabric_attr->prov_name, prov_names->names[i], strlen(prov_names->names[i])) == 0) {
                        DEBUG_MSG("Matched Provider (%s): %s %s", prov_names->names[i], prov_cur->fabric_attr->prov_name, prov_cur->domain_attr->name);
                        prov_found = prov_cur;
                        if (domain_names == NULL) {
                            rofi->info = fi_dupinfo(prov_cur);
                            return;
                        }
                        else {
                            for (int j = 0; j < domain_names->num; j++) {
                                DEBUG_MSG("checking Domain (%s): %s %s", domain_names->names[j], prov_cur->fabric_attr->prov_name, prov_cur->domain_attr->name);
                                if (strncmp(prov_cur->domain_attr->name, domain_names->names[j], strlen(domain_names->names[j])) == 0) {
                                    DEBUG_MSG("Matched Domain (%s): %s %s", domain_names->names[j], prov_cur->fabric_attr->prov_name, prov_cur->domain_attr->name);
                                    rofi->info = fi_dupinfo(prov_cur);
                                    return;
                                }
                            }
                            DEBUG_MSG("No matching domain found for provider (%s): %s %s looking at next provider", prov_names->names[i], prov_cur->fabric_attr->prov_name, prov_cur->domain_attr->name);
                        }
                    }
                }
                if (prov_found) {
                    WARN_MSG("Found provider without matching domain, using default domain for provider : %s %s", prov_found->fabric_attr->prov_name, prov_found->domain_attr->name);
                    rofi->info = fi_dupinfo(prov_found);
                    return;
                }
            }
            else {
                if (domain_names) {
                    for (int j = 0; j < domain_names->num; j++) {
                        DEBUG_MSG("checking Domain (%s): %s %s", domain_names->names[j], prov_cur->fabric_attr->prov_name, prov_cur->domain_attr->name);
                        if (strncmp(prov_cur->domain_attr->name, domain_names->names[j], strlen(domain_names->names[j])) == 0) {
                            DEBUG_MSG("Matched Domain (%s): %s %s", domain_names->names[j], prov_cur->fabric_attr->prov_name, prov_cur->domain_attr->name);
                            rofi->info = fi_dupinfo(prov_cur);
                            return;
                        }
                    }
                }
            }
            prov_cur = prov_cur->next;
        }
    }
}

int rofi_transport_init(struct fi_info *hints, rofi_transport_t *rofi, rofi_names_t *prov_names, rofi_names_t *domain_names) {
    int ofi_version_major = 0;
    int ofi_version_minor = 0;

    // The libfabric version should be set during configure time and exported to 
    // the compiler via __OFI_VERSION__. To allow for preprocessor boolean 
    // checks, major and minor versions are combined according to 
    // (major * 100 + minor). Here we extract the major and minor versions for 
    // later use by fi_getinfo.
#ifndef __OFI_VERSION__
    ERR_MSG("__OFI_VERSION__ was not defined at compile time!");
    abort();
#else
    ofi_version_major = __OFI_VERSION__ / 100;
    ofi_version_minor = __OFI_VERSION__ - (ofi_version_major * 100);
    DEBUG_MSG("ROFI compiled for use with libfabric %d.%d\n", ofi_version_major, ofi_version_minor);
    
#endif

    DEBUG_MSG("fi_getinfo");
    struct fi_info *prov = fi_allocinfo();
    //int ret = fi_getinfo(ROFI_FI_VERSION, NULL, NULL, 0, hints, &prov);
    int ret = fi_getinfo(FI_VERSION(ofi_version_major, ofi_version_minor), NULL, NULL, 0, hints, &prov);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_getinfo", ret);
    }    

#ifdef _DEBUG
    struct fi_info *prov_cur = prov; // rofi->info;
    while (prov_cur != NULL) {
        DEBUG_MSG("Available Provider: %s  Version: (%u.%u) Fabric: %s Domain: %s max_inject: %zu, max_msg: %zu, stx: %s, MR_RMA_EVENT: %s, msg: %s, rma: %s, read: %s, write: %s, remote_read: %s, remote_write: %s, rma_event: %s, atomic: %s, collective: %s",
                  prov_cur->fabric_attr->prov_name,
                  FI_MAJOR(prov_cur->fabric_attr->prov_version),
                  FI_MINOR(prov_cur->fabric_attr->prov_version),
                  prov_cur->fabric_attr->name,
                  prov_cur->domain_attr->name,
                  prov_cur->tx_attr->inject_size,
                  prov_cur->ep_attr->max_msg_size,
                  prov_cur->domain_attr->max_ep_stx_ctx == 0 ? "no" : "yes",
                  prov_cur->domain_attr->mr_mode & FI_MR_RMA_EVENT ? "yes" : " no",
                  prov_cur->caps & FI_MSG ? "yes" : "no",
                  prov_cur->caps & FI_RMA ? "yes" : "no",
                  prov_cur->caps & FI_READ ? "yes" : "no",
                  prov_cur->caps & FI_WRITE ? "yes" : "no",
                  prov_cur->caps & FI_REMOTE_READ ? "yes" : "no",
                  prov_cur->caps & FI_REMOTE_WRITE ? "yes" : "no",
                  prov_cur->caps & FI_RMA_EVENT ? "yes" : "no",
                  prov_cur->caps & FI_ATOMIC ? "yes" : "no",
                  prov_cur->caps & FI_COLLECTIVE ? "yes" : "no");
        prov_cur = prov_cur->next;
    }
#endif


    rofi_transport_select_provider(prov, rofi, prov_names, domain_names);
    

    if (rofi->info == NULL) {
        rofi_transport_select_provider(prov, rofi, NULL, NULL);
    }
    fi_freeinfo(prov);

    DEBUG_MSG("Selected Provider: %s  Version: (%u.%u) Fabric: %s Domain: %s max_inject: %zu, max_msg: %zu, stx: %s, MR_RMA_EVENT: %s, msg: %s, rma: %s, read: %s, write: %s, remote_read: %s, remote_write: %s, rma_event: %s, atomic: %s, collective: %s",
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
              rofi->info->caps & FI_COLLECTIVE ? "yes" : "no");

    if (rofi->info == NULL) {
        ERR_MSG("Error initializing ROFI. No matching provider found. Aborting.");
        return -1;
    }

    DEBUG_MSG("rofi->info: %p, provider: %s, caps: 0x%lx\n",
       rofi->info, rofi->info->fabric_attr->prov_name, rofi->info->caps);

    DEBUG_MSG("Selected provider: %s\n", rofi->info->fabric_attr->prov_name);
    DEBUG_MSG("Selected caps: 0x%lx\n", rofi->info->caps);

    if(rofi->info->caps & FI_ATOMIC) {
        DEBUG_MSG("Selected atomic: yes");
    }
    else {
        DEBUG_MSG("Selected atomic: no");
    } 

#ifdef __OFI_PROV_CXI__
    // The CXI tests in libfabric 2.1 follow up the selection of the 
    // CXI provider with some additional adjustments to the fi_info struct 
    // before initializing the fabric, creating the endpoint, etc.
    // It is not clear they are necessary but they are repeated here.
    // Before these adjustments, fi_getinfo will return cxi providers with all of 
    // the capabilities (including RMA) except FI_SOURCE and FI_SOURCE_ERR
    // Note: CXI man pages for 2.3.1 suggest not turning on FI_SOURCE/FI_SOURCE_ERR
    rofi->info->ep_attr->tx_ctx_cnt = rofi->info->domain_attr->tx_ctx_cnt;
    rofi->info->ep_attr->rx_ctx_cnt = rofi->info->domain_attr->rx_ctx_cnt;
//    rofi->info->caps |= FI_SOURCE | FI_SOURCE_ERR;
//    rofi->info->rx_attr->caps |= FI_SOURCE | FI_SOURCE_ERR;
#endif

    ret = rofi_transport_init_fabric_resources(rofi);
    if (ret) {
        // already would have printed the error.
        return ret;
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
        rofi->fi_collective = 0;
    }
    else {
        rofi->fi_collective = FI_COLLECTIVE;
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
    char buf[256];
    size_t buflen = 256;
    DEBUG_MSG("epname: %s", fi_av_straddr(rofi->av, epname, buf, &buflen));
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
    struct fi_cntr_attr send_cntr_attr = {0};
    struct fi_cntr_attr recv_cntr_attr = {0};
    put_cntr_attr.events = FI_CNTR_EVENTS_COMP;
    get_cntr_attr.events = FI_CNTR_EVENTS_COMP;
    send_cntr_attr.events = FI_CNTR_EVENTS_COMP;
    recv_cntr_attr.events = FI_CNTR_EVENTS_COMP;
    put_cntr_attr.wait_obj = FI_WAIT_UNSPEC;
    get_cntr_attr.wait_obj = FI_WAIT_UNSPEC;
    send_cntr_attr.wait_obj = FI_WAIT_UNSPEC;
    recv_cntr_attr.wait_obj = FI_WAIT_UNSPEC;

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

    DEBUG_MSG("send FI_CNTR_OPEN");
    ret = fi_cntr_open(rofi->domain, &send_cntr_attr, &rofi->send_cntr, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_cntr_open", ret);
        return ret;
    }

    DEBUG_MSG("recv FI_CNTR_OPEN");
    ret = fi_cntr_open(rofi->domain, &recv_cntr_attr, &rofi->recv_cntr, NULL);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_cntr_open", ret);
        return ret;
    }

    struct fi_cq_attr cq_attr = {0};
    cq_attr.format = FI_CQ_FORMAT_CONTEXT;
    cq_attr.wait_obj = FI_WAIT_UNSPEC;

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

    // The original verbs code (in the #else branch, below) makes adjustments to the 
    // fi_info struct containing information for the selected provider at this point. This is not 
    // required for CXI, so this directive removes it for CXI.
#ifdef __OFI_PROV_CXI__
    ;
#else
    rofi->info->ep_attr->tx_ctx_cnt = 0;
    rofi->info->caps = FI_RMA | FI_WRITE | FI_READ | FI_REMOTE_WRITE | FI_REMOTE_READ | FI_ATOMIC | rofi->fi_collective;
    rofi->info->tx_attr->op_flags = FI_DELIVERY_COMPLETE; // FI_TRANSMIT_COMPLETE fails, FI_DELIVERY_COMPLETE works but I dont see a difference?
    rofi->info->mode = 0;
    rofi->info->tx_attr->mode = 0;
    rofi->info->rx_attr->mode = 0;
    rofi->info->rx_attr->size = 1024;
    rofi->info->tx_attr->size = 1024;
    rofi->info->tx_attr->caps = rofi->info->caps;
    rofi->info->rx_attr->caps = FI_RECV | rofi->fi_collective; // to drive progress
#endif

    DEBUG_MSG("rofi->info: %p, provider: %s, caps: 0x%lx\n",
        rofi->info, rofi->info->fabric_attr->prov_name, rofi->info->caps);

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
    ret = fi_ep_bind(rofi->ep, &rofi->put_cntr->fid, FI_WRITE ); // we dont include FI_REMOTE_WRITE as this would update the counter whenever a remote request comes in, i.e. we only care about local requests
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_ep_bind put_cntr", ret);
        return ret;
    }

    // bind get cntr
    DEBUG_MSG("FI_EP_BIND get_cntr");
    ret = fi_ep_bind(rofi->ep, &rofi->get_cntr->fid, FI_READ );// we dont include FI_REMOTE_READ as this would update the counter whenever a remote request comes in, i.e. we only care about local requests
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_ep_bind get_cntr", ret);
        return ret;
    }

    // bind send cntr
    DEBUG_MSG("FI_EP_BIND send_cntr");
    ret = fi_ep_bind(rofi->ep, &rofi->send_cntr->fid, FI_SEND);
    if (ret) {
        ROFI_TRANSPORT_ERR_MSG("fi_ep_bind send_cntr", ret);
        return ret;
    }

    // bind recv cntr
    DEBUG_MSG("FI_EP_BIND recv_cntr");
    ret = fi_ep_bind(rofi->ep, &rofi->recv_cntr->fid, FI_RECV);
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
        char buf[256];
        size_t buflen = 256;
        int ret = rt_get(i, "epname", addr_ptr, rofi->desc.addrlen);
        DEBUG_MSG("Got EP address name from %i (%s).", i, fi_av_straddr(rofi->av, addr_ptr, buf, &buflen));
        if (ret) {
            ERR_MSG("Error getting EP address name from %i (%d).", i, ret);
            free(all_addrs);
            return ret;
        }
    }

    DEBUG_MSG("FI_AV_INSERT");
    int ret = fi_av_insert(rofi->av, all_addrs, rofi->desc.nodes, rofi->remote_addrs, 0, NULL);
    if (ret < 0) {
        ROFI_TRANSPORT_ERR_MSG("ft_av_insert", ret);
        free(all_addrs);
        return ret;
    }
    else if (ret != rofi->desc.nodes) {
        ERR_MSG("fi_av_insert: number of addresses inserted = %d;"
                " number of addresses given = %d\n",
                ret, rofi->desc.nodes);
        free(all_addrs);
        return ret;
    }
    free(all_addrs);
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
    int err =ret;
    if (ret == 1) {
        WARN_MSG("unexpected cq event"); // warnd
    }
    else if (ret < 0 && ret != -FI_EAGAIN) {
        ROFI_TRANSPORT_ERR_MSG("rofi_transport_progress", ret);
        do {
            struct fi_cq_err_entry ebuf = {0};
            int ret = fi_cq_readerr(rofi->cq, (void *)&ebuf, 0);
            if (ret > 0) {
                const char *errmsg = fi_cq_strerror(rofi->cq, ebuf.prov_errno, ebuf.err_data, NULL, 0);
                const char *errmsg1 = fi_cq_strerror(rofi->cq, ebuf.err, ebuf.err_data, NULL, 0);
#if __OFI_VERSION__ >= 120
                ERR_MSG("ret: %d context: %p flags %llu len: %d buf: %p data: %llu tag %llu olen %llu err %d prov_err %d err_data %p err_data_size %llu src_addr %d %s %s", ret,
                            ebuf.op_context,ebuf.flags,ebuf.len,ebuf.buf,ebuf.data, ebuf.tag,ebuf.olen,ebuf.err,ebuf.prov_errno,ebuf.err_data,ebuf.err_data_size,ebuf.src_addr,errmsg, errmsg1);
#else
                ERR_MSG("ret: %d context: %p flags %llu len: %d buf: %p data: %llu tag %llu olen %llu err %d prov_err %d err_data %p err_data_size %llu %s %s", ret,
                            ebuf.op_context,ebuf.flags,ebuf.len,ebuf.buf,ebuf.data, ebuf.tag,ebuf.olen,ebuf.err,ebuf.prov_errno,ebuf.err_data,ebuf.err_data_size,errmsg, errmsg1);
#endif
                err = ebuf.err;
            }
            else if (ret < 0) {
                ROFI_TRANSPORT_ERR_MSG("fi_cq_readerr", ret);
                return ret;
            }
        } while (ret == 1);
        return (err);
    }
    return 0;
}

// checks the rma return status, and aborts if not -FI_EAGAIN
// note this does not try to make progress
int rofi_transport_locked_ctx_check_err(rofi_transport_t *rofi, int err, struct fid_cntr *cntr) {
    int err_cnt = fi_cntr_readerr(cntr);

    if (err_cnt > rofi->error_cnt){
        DEBUG_MSG(" found  %lu  errors cnts... ", err_cnt);
        rofi->error_cnt = err_cnt;
        if (err == -FI_EAGAIN) {
            for (int j=0;j<err_cnt;j++){
                struct fi_cq_err_entry ebuf = {0};
                int ret = fi_cq_readerr(rofi->cq, (void *)&ebuf, 0);
                const char *errmsg = fi_cq_strerror(rofi->cq, ebuf.prov_errno, ebuf.err_data, NULL, 0);
                const char *errmsg1 = fi_cq_strerror(rofi->cq, ebuf.err, ebuf.err_data, NULL, 0);
// the src_addr field was added sometime around libfabric 1.20
#if __OFI_VERSION__ >= 120
                ERR_MSG("ret: %d context: %p flags %llu len: %d buf: %p data: %llu tag %llu olen %llu err %d prov_err %d err_data %p err_data_size %llu src_addr %d %s %s", ret,
                            ebuf.op_context,ebuf.flags,ebuf.len,ebuf.buf,ebuf.data, ebuf.tag,ebuf.olen,ebuf.err,ebuf.prov_errno,ebuf.err_data,ebuf.err_data_size,ebuf.src_addr,errmsg, errmsg1);
#else
                ERR_MSG("ret: %d context: %p flags %llu len: %d buf: %p data: %llu tag %llu olen %llu err %d prov_err %d err_data %p err_data_size %llu %s %s", ret,
                            ebuf.op_context,ebuf.flags,ebuf.len,ebuf.buf,ebuf.data, ebuf.tag,ebuf.olen,ebuf.err,ebuf.prov_errno,ebuf.err_data,ebuf.err_data_size,errmsg, errmsg1);
#endif

                // struct fi_cq_err_entry ebuf = {0};
                // int ret = fi_cq_readerr(rofi->cq, (void *)&ebuf, 0);
                // if (ret > 0 && ebuf.err == -FI_EACCES) {
                //     const char *errmsg = fi_cq_strerror(rofi->cq, ebuf.prov_errno, ebuf.err_data, NULL, 0);
                //     const char *errmsg1 = fi_cq_strerror(rofi->cq, ebuf.err, ebuf.err_data, NULL, 0);
                //     ERR_MSG("Error: %s %s %d %d \n", errmsg, ebuf.prov_errno, ebuf.err);
                    
                //     return ret;
                // }
                // else if (ret < 0 && ret != -FI_EAGAIN) {
                //     ROFI_TRANSPORT_ERR_MSG("fi_cq_readerr", ret);
                //     return ret;
                // }
            }
            return err;
        }
        else if (err) {
            for (int j=0;j<err_cnt;j++){
                struct fi_cq_err_entry ebuf = {0};
                int ret = fi_cq_readerr(rofi->cq, (void *)&ebuf, 0);
                const char *errmsg = fi_cq_strerror(rofi->cq, ebuf.prov_errno, ebuf.err_data, NULL, 0);
                const char *errmsg1 = fi_cq_strerror(rofi->cq, ebuf.err, ebuf.err_data, NULL, 0);
#if __OFI_VERSION__ >= 120
                ERR_MSG("ret: %d context: %p flags %llu len: %d buf: %p data: %llu tag %llu olen %llu err %d prov_err %d err_data %p err_data_size %llu src_addr %d %s %s", ret,
                            ebuf.op_context,ebuf.flags,ebuf.len,ebuf.buf,ebuf.data, ebuf.tag,ebuf.olen,ebuf.err,ebuf.prov_errno,ebuf.err_data,ebuf.err_data_size,ebuf.src_addr,errmsg,errmsg1);
#else
                ERR_MSG("ret: %d context: %p flags %llu len: %d buf: %p data: %llu tag %llu olen %llu err %d prov_err %d err_data %p err_data_size %llu %s %s", ret,
                            ebuf.op_context,ebuf.flags,ebuf.len,ebuf.buf,ebuf.data, ebuf.tag,ebuf.olen,ebuf.err,ebuf.prov_errno,ebuf.err_data,ebuf.err_data_size,errmsg, errmsg1);
#endif
            }
            return err;
        }
    }
    return 0;
}

// checks the rma return status, and aborts if not -FI_EAGAIN
// note this does not try to make progress
int rofi_transport_ctx_check_err(rofi_transport_t *rofi, int err) {
  
    if (err == -FI_EAGAIN || err == -FI_EAVAIL) {
        struct fi_cq_err_entry ebuf = {0};
        pthread_mutex_lock(&rofi->lock);
        int ret = fi_cq_readerr(rofi->cq, (void *)&ebuf, 0);
        pthread_mutex_unlock(&rofi->lock);
        if (ret > 0 && ebuf.err == -FI_EACCES) {
            const char *errmsg = fi_cq_strerror(rofi->cq, ebuf.prov_errno, ebuf.err_data, NULL, 0);
            ERR_MSG("Error: %s %d %d \n", errmsg, ebuf.prov_errno, ebuf.err);
            
            return ret;
        }
        else if (ret < 0 && ret != -FI_EAGAIN) {
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
        if (ret > 0 && ebuf.err == -FI_EACCES) {
            const char *errmsg = fi_cq_strerror(rofi->cq, ebuf.prov_errno, ebuf.err_data, NULL, 0);
            ERR_MSG("Error: %s %d %d \n", errmsg, ebuf.prov_errno, ebuf.err);
            
            return ret;
        }
        else if (ret < 0 && ret != -FI_EAGAIN) {
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

int rofi_transport_locked_wait_on_cntr(rofi_transport_t *rofi, _Atomic uint64_t *pending_cntr, struct fid_cntr *cntr) {
     uint64_t prev_expected_cnt = atomic_load_explicit(pending_cntr, memory_order_relaxed);
    uint64_t old_cnt = fi_cntr_read(cntr);
    uint64_t expected_cnt = atomic_load_explicit(pending_cntr, memory_order_relaxed);
    uint64_t err_cnt = fi_cntr_readerr(cntr);
    uint64_t cur_cnt = old_cnt;

    DEBUG_MSG("Before Waiting for  %lu  cnts... cur_cnt: %lu err_cnt: %lu expected_cnt: %lu prev_expected_cnt: %lu", expected_cnt, cur_cnt, err_cnt, expected_cnt, prev_expected_cnt);

    
    while (cur_cnt < expected_cnt || prev_expected_cnt < expected_cnt || cur_cnt != old_cnt) {
        DEBUG_MSG("Waiting for  %lu  cnts... cur_cnt: %lu err_cnt: %lu expected_cnt: %lu prev_expected_cnt: %lu", expected_cnt, cur_cnt, err_cnt, expected_cnt, prev_expected_cnt);

        prev_expected_cnt = expected_cnt;
        old_cnt = cur_cnt;
        int ret = rofi_transport_progress(rofi);
        ret = fi_cntr_wait(cntr, prev_expected_cnt, -1);
        
        
        if (ret != -FI_ETIMEDOUT){
            int ret = rofi_transport_progress(rofi);
            if (ret) {
                DEBUG_MSG("rofi_transport_progress error!!!");
                return ret;
            }
            ret = rofi_transport_locked_ctx_check_err(rofi, ret, cntr);
            DEBUG_MSG("Checking expected_cnt: %lu prev_expected_cnt: %lu old_cnt: %lu cur_cnt: %lu expected_cnt: %lu err_cnt: %lu ",expected_cnt, prev_expected_cnt, old_cnt, cur_cnt, expected_cnt, rofi->error_cnt);
        }
        cur_cnt = fi_cntr_read(cntr);
        expected_cnt = atomic_load_explicit(pending_cntr, memory_order_relaxed); // this could be updated by another thread
    } 
    assert(prev_expected_cnt <= expected_cnt);
    return 0;
}

int rofi_transport_wait_on_cntr(rofi_transport_t *rofi, _Atomic uint64_t *pending_cntr, struct fid_cntr *cntr) {
    uint64_t prev_expected_cnt = atomic_load_explicit(pending_cntr, memory_order_relaxed);
    pthread_mutex_lock(&rofi->lock);
    uint64_t old_cnt = fi_cntr_read(cntr);
    uint64_t expected_cnt = atomic_load_explicit(pending_cntr, memory_order_relaxed);
    uint64_t err_cnt = fi_cntr_readerr(cntr);
    // pthread_mutex_unlock(&rofi->lock);
    uint64_t cur_cnt = old_cnt;

    DEBUG_MSG("Before Waiting on cntr_addr: [%p, %p] for  %lu  cnts... cur_cnt: %lu err_cnt: %lu expected_cnt: %lu prev_expected_cnt: %lu", cntr, pending_cntr, expected_cnt, cur_cnt, err_cnt, expected_cnt, prev_expected_cnt);

    
    while (cur_cnt < expected_cnt || prev_expected_cnt < expected_cnt || cur_cnt != old_cnt) {
        DEBUG_MSG("Waiting for  %lu  cnts... cur_cnt: %lu err_cnt: %lu expected_cnt: %lu prev_expected_cnt: %lu", expected_cnt, cur_cnt, err_cnt, expected_cnt, prev_expected_cnt);

        prev_expected_cnt = expected_cnt;
        old_cnt = cur_cnt;
        // pthread_mutex_lock(&rofi->lock);
        int ret = rofi_transport_progress(rofi);
        ret = fi_cntr_wait(cntr, prev_expected_cnt, -1);
        
        
        if (ret != -FI_ETIMEDOUT){
            int ret = rofi_transport_progress(rofi);
            if (ret) {
                DEBUG_MSG("rofi_transport_progress error!!!");
                pthread_mutex_unlock(&rofi->lock);
                return ret;
            }
            ret = rofi_transport_locked_ctx_check_err(rofi, ret, cntr);
            DEBUG_MSG("Checking expected_cnt: %lu prev_expected_cnt: %lu old_cnt: %lu cur_cnt: %lu expected_cnt: %lu err_cnt: %lu ",expected_cnt, prev_expected_cnt, old_cnt, cur_cnt, expected_cnt, rofi->error_cnt);
        }
        cur_cnt = fi_cntr_read(cntr);
        // pthread_mutex_unlock(&rofi->lock);
        expected_cnt = atomic_load_explicit(pending_cntr, memory_order_relaxed); // this could be updated by another thread
    } 
    pthread_mutex_unlock(&rofi->lock);
    assert(prev_expected_cnt <= expected_cnt);
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

    DEBUG_MSG("Waiting on context comp %p", context);

    while (true) {
        int ret = fi_cq_read(rofi->cq, &buf, 1);
        if (ret < 0 && ret != -FI_EAGAIN) {
            if (ret == -FI_EAVAIL) {
                ret = fi_cq_readerr(rofi->cq, &err_entry, 0);
                if (ret > 0 && err_entry.err == -FI_EACCES) {
                    const char *errmsg = fi_cq_strerror(rofi->cq, err_entry.prov_errno, err_entry.err_data, NULL, 0);
                    ERR_MSG("Error: %s %d %d\n", errmsg, err_entry.prov_errno, err_entry.err);
                    
                    return ret;
                }
            }
            else{
                ROFI_TRANSPORT_ERR_MSG("fi_cq_read", ret);
                return ret;
            }
        }
        if (buf.op_context && buf.op_context == context) {
            return 0;
        }
        else if (buf.op_context) {
            DEBUG_MSG("Unexpected context comp %p != %p", buf.op_context, context);
        }
    }
}

int rofi_transport_put_inject(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, fi_addr_t pe, const void *src_addr, size_t len) {
    pthread_mutex_lock(&rofi->lock);
    atomic_max_u64(&rofi->pending_put_cntr, fi_cntr_read(rofi->put_cntr));

    DEBUG_MSG("fi_inject_write %p %p %d %d %p 0x%lx pending_put_cntr=%lu", rofi->ep, src_addr, len, pe, rma_iov->addr, rma_iov->key, rofi->pending_put_cntr);
    int ret = fi_inject_write(rofi->ep, src_addr, len, pe, rma_iov->addr, rma_iov->key);
    while (ret) { // retry while FI_EAGAIN
        ret = rofi_transport_check_rma_err(rofi, ret);
        if (ret) {
            pthread_mutex_unlock(&rofi->lock);
            return ret;
        }
        ret = fi_inject_write(rofi->ep, src_addr, len, pe, rma_iov->addr, rma_iov->key);
    }
    atomic_fetch_add_explicit(&rofi->pending_put_cntr, 1, memory_order_relaxed); // ensure visibility
    pthread_mutex_unlock(&rofi->lock);
    DEBUG_MSG("fi_inject_write done %p %p %d %d %p 0x%lx pending_put_cntr=%lu", rofi->ep, src_addr, len, pe, rma_iov->addr, rma_iov->key, rofi->pending_put_cntr);
    return 0;
}

int rofi_transport_put_large(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, fi_addr_t pe, const void *src_addr, size_t len, void *desc, void *context) {

    uint8_t *src_cur_addr = (uint8_t *)src_addr;
    uint8_t *src_end_addr = src_cur_addr + len;
    uint64_t dst_cur_addr = (uint64_t)rma_iov->addr;
    pthread_mutex_lock(&rofi->lock);
    
    atomic_max_u64(&rofi->pending_put_cntr, fi_cntr_read(rofi->put_cntr));
    DEBUG_MSG("fi_write %p %p %lu %d %p 0x%lx pending_put_cntr=%lu", rofi->ep, src_cur_addr, (unsigned long)len, pe, rma_iov->addr, rma_iov->key, rofi->pending_put_cntr);
    while (src_cur_addr < src_end_addr) {
        uint64_t cur_len = MIN(src_end_addr - src_cur_addr, rofi->desc.max_message_size);
        

        int ret = fi_write(rofi->ep, src_cur_addr, cur_len, desc, pe, dst_cur_addr, rma_iov->key, context);

        while (ret) { // retry while FI_EAGAIN
            ret = rofi_transport_check_rma_err(rofi, ret);
            if (ret) {
                pthread_mutex_unlock(&rofi->lock);
                return ret;
            }
            ret = fi_write(rofi->ep, src_cur_addr, cur_len, desc, pe, dst_cur_addr, rma_iov->key, context);
        }
        atomic_fetch_add_explicit(&rofi->pending_put_cntr, 1, memory_order_relaxed);
        src_cur_addr += cur_len;
        dst_cur_addr += cur_len;
    }
    pthread_mutex_unlock(&rofi->lock);
    DEBUG_MSG("fi_write %p %p %lu %d %p 0x%lx pending_put_cntr=%lu", rofi->ep, src_addr, (unsigned long)len, pe, rma_iov->addr, rma_iov->key, rofi->pending_put_cntr);
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
    atomic_max_u64(&rofi->pending_get_cntr, fi_cntr_read(rofi->get_cntr));
    int ret = fi_read(rofi->ep, dst_addr, len, desc, pe, rma_iov->addr, rma_iov->key, context);
    while (ret) { // retry while FI_EAGAIN
        ret = rofi_transport_check_rma_err(rofi, ret);
        if (ret) {
            return ret;
        }
        ret = fi_read(rofi->ep, dst_addr, len, desc, pe, rma_iov->addr, rma_iov->key, context);
    }
    atomic_fetch_add_explicit(&rofi->pending_get_cntr, 1, memory_order_relaxed);
    pthread_mutex_unlock(&rofi->lock);
    return 0;
}

int rofi_transport_get_large(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, void *dst_addr, size_t len, void *desc, void *context) {
    uint64_t src_cur_addr = (uint64_t)rma_iov->addr;

    uint8_t *dst_cur_addr = (uint8_t *)dst_addr;
    uint8_t *dst_end_addr = dst_cur_addr + len;
    pthread_mutex_lock(&rofi->lock);
    atomic_max_u64(&rofi->pending_get_cntr, fi_cntr_read(rofi->get_cntr));
    while (dst_cur_addr < dst_end_addr) {
        uint64_t cur_len = MIN(dst_end_addr - dst_cur_addr, rofi->desc.max_message_size);
        int ret = fi_read(rofi->ep, dst_cur_addr, cur_len, desc, pe, src_cur_addr, rma_iov->key, context);
        while (ret) { // retry while FI_EAGAIN
            ret = rofi_transport_check_rma_err(rofi, ret);
            if (ret) {
                return ret;
            }
            ret = fi_read(rofi->ep, dst_cur_addr, cur_len, desc, pe, src_cur_addr, rma_iov->key, context);
            
        }
        atomic_fetch_add_explicit(&rofi->pending_get_cntr, 1, memory_order_relaxed);
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

int rofi_transport_atomic(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, const void *value, size_t count,
                          enum fi_datatype datatype, enum fi_op op, void *value_desc, void *context) {
    pthread_mutex_lock(&rofi->lock);
    atomic_max_u64(&rofi->pending_put_cntr, fi_cntr_read(rofi->put_cntr));

    int ret = fi_atomic(rofi->ep, value, count, value_desc, rofi->remote_addrs[pe], rma_iov->addr, rma_iov->key,
                        datatype, op, context);
    while (ret) {
        ret = rofi_transport_check_rma_err(rofi, ret);
        if (ret) {
            pthread_mutex_unlock(&rofi->lock);
            return ret;
        }
        ret = fi_atomic(rofi->ep, value, count, value_desc, rofi->remote_addrs[pe], rma_iov->addr, rma_iov->key,
                        datatype, op, context);
    }
    atomic_fetch_add_explicit(&rofi->pending_put_cntr, 1, memory_order_relaxed);
    pthread_mutex_unlock(&rofi->lock);
    return 0;
}

int rofi_transport_atomic_fetch(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, const void *value, void *result,
                                size_t count, enum fi_datatype datatype, enum fi_op op, void *value_desc,
                                void *result_desc, void *context) {
    pthread_mutex_lock(&rofi->lock);
    atomic_max_u64(&rofi->pending_get_cntr, fi_cntr_read(rofi->get_cntr));

    int ret = fi_fetch_atomic(rofi->ep, value, count, value_desc, result, result_desc, rofi->remote_addrs[pe],
                              rma_iov->addr, rma_iov->key, datatype, op, context);
    while (ret) {
        ret = rofi_transport_check_rma_err(rofi, ret);
        if (ret) {
            pthread_mutex_unlock(&rofi->lock);
            return ret;
        }
        ret = fi_fetch_atomic(rofi->ep, value, count, value_desc, result, result_desc, rofi->remote_addrs[pe],
                              rma_iov->addr, rma_iov->key, datatype, op, context);
    }
    atomic_fetch_add_explicit(&rofi->pending_get_cntr, 1, memory_order_relaxed);
    pthread_mutex_unlock(&rofi->lock);
    return 0;
}

int rofi_transport_compare_atomic(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, const void *value,
                                  const void *compare, void *result, size_t count, enum fi_datatype datatype, enum fi_op op,
                                  void *value_desc, void *compare_desc, void *result_desc, void *context) {
    pthread_mutex_lock(&rofi->lock);
    atomic_max_u64(&rofi->pending_get_cntr, fi_cntr_read(rofi->get_cntr) );

    int ret = fi_compare_atomic(rofi->ep, value, count, value_desc, compare, compare_desc, result, result_desc,
                                rofi->remote_addrs[pe], rma_iov->addr, rma_iov->key, datatype, op, context);
    while (ret) {
        ret = rofi_transport_check_rma_err(rofi, ret);
        if (ret) {
            pthread_mutex_unlock(&rofi->lock);
            return ret;
        }
        ret = fi_compare_atomic(rofi->ep, value, count, value_desc, compare, compare_desc, result, result_desc,
                                rofi->remote_addrs[pe], rma_iov->addr, rma_iov->key, datatype, op, context);
    }
    atomic_fetch_add_explicit(&rofi->pending_get_cntr, 1, memory_order_relaxed);
    pthread_mutex_unlock(&rofi->lock);
    return 0;
}

int rofi_transport_send(rofi_transport_t *rofi, void *buf, size_t len, uint64_t pe) {
    pthread_mutex_lock(&rofi->lock);
    uint64_t finish_flag = 0;
    atomic_fetch_add_explicit(&rofi->pending_send_cntr, 1, memory_order_relaxed);
    int ret = fi_send(rofi->ep, buf, len, NULL, rofi->remote_addrs[pe], &finish_flag);
    while (ret) { // retry while FI_EAGAIN
        ret = rofi_transport_check_rma_err(rofi, ret);
        if (ret) {
            pthread_mutex_unlock(&rofi->lock);
            return ret;
        }
        ret = fi_send(rofi->ep, buf, len, NULL, rofi->remote_addrs[pe], &finish_flag);
    }
    rofi_transport_locked_wait_on_cntr(rofi, &rofi->pending_send_cntr, rofi->send_cntr);
    pthread_mutex_unlock(&rofi->lock);

    return 0;
}

// we actually need to make this async, so store the finish flag in a hashmap, that we can use to check on subsequent calls
int rofi_transport_recv(rofi_transport_t *rofi, void *buf, size_t len) {
    pthread_mutex_lock(&rofi->lock);
    uint64_t finish_flag = 0;
    atomic_fetch_add_explicit(&rofi->pending_recv_cntr, 1, memory_order_relaxed);
    int ret = fi_recv(rofi->ep, buf, len, NULL, 0, &finish_flag);
    while (ret) { // retry while FI_EAGAIN
        ret = rofi_transport_check_rma_err(rofi, ret);
        if (ret) {
            pthread_mutex_unlock(&rofi->lock);
            return ret;
        }
        ret = fi_recv(rofi->ep, buf, len, NULL, 0, &finish_flag);
    }
    rofi_transport_locked_wait_on_cntr(rofi, &rofi->pending_recv_cntr, rofi->recv_cntr);
    pthread_mutex_unlock(&rofi->lock);

    return 0;
}

// New function to be used only during rofi_init_internal to exchange addressing information 
// for the rofi.mr used for subsequent exchanges, barriers, etc.
int rofi_transport_exchange_init_mr_info(rofi_transport_t *rofi, rofi_mr_desc *mr) {

    struct fi_rma_iov rma_iov;
    rma_iov.addr = (uint64_t)mr->start;
    rma_iov.key = fi_mr_key(mr->fid);
    DEBUG_MSG("Exchanging initialization (rofi.mr) MR Info (key: 0x%lx, addr: 0x%lx)....", rma_iov.key, rma_iov.addr);

    int ret = rt_exchange_data("mr_init_info", &rma_iov, sizeof(struct fi_rma_iov), mr->iov, rofi->desc.nid, rofi->desc.nodes);
    if (ret) {
        ERR_MSG("Error exchanging info for memory region alloc buffer. Aborting!");
        return ret;
    }

#ifdef _DEBUG
    for (int i = 0; i < rofi->desc.nodes; i++) {
        DEBUG_MSG("\t Results of exchanging inital MR info: Node: %d Key: 0x%lx Addr: 0x%lx", i, mr->iov[i].key, mr->iov[i].addr);
    }
#endif
    return 0;
}

int rofi_transport_exchange_mr_info(rofi_transport_t *rofi, rofi_mr_desc *mr) {
    if (rofi->desc.nodes == 1) {
        return 0;
    }

    //create an array from 0..num_pes-1 to represent all PEs for the sub barrier
    uint64_t *pes = (uint64_t *)malloc(rofi->desc.nodes * sizeof(uint64_t));
    for (uint64_t i = 0; i < rofi->desc.nodes; i++) {
        pes[i] = i;
    }

    int ret = rofi_transport_sub_exchange_mr_info(rofi, mr, pes, rofi->desc.nodes);
    free(pes);
    return ret;


//     struct fi_rma_iov rma_iov;
//     rma_iov.addr = (uint64_t)mr->start;
//     rma_iov.key = fi_mr_key(mr->fid);
//     DEBUG_MSG("Exchanging MR Info (key: 0x%lx, addr: 0x%lx)....", rma_iov.key, rma_iov.addr);

//     int ret = rt_exchange_data("mr_info", &rma_iov, sizeof(struct fi_rma_iov), mr->iov, rofi->desc.nid, rofi->desc.nodes);
//     if (ret) {
//         ERR_MSG("Error exchanging info for memory region alloc buffer. Aborting!");
//         return ret;
//     }

// #ifdef _DEBUG
//     for (int i = 0; i < rofi->desc.nodes; i++) {
//         DEBUG_MSG("\t Node: %d Key: 0x%lx Addr: 0x%lx", i, mr->iov[i].key, mr->iov[i].addr);
//     }
// #endif
//     return 0;
}

// for use when FI_COLLECTIVE not available
int rofi_transport_sub_exchange_mr_info_manual(rofi_transport_t *rofi, rofi_mr_desc *mr, uint64_t *pes, uint64_t num_pes) {
    int global_me = rofi->desc.nid;
    int team_me = global_me;
    if (pes != NULL) { // doing sub barrier, figure out team pe id
        for (int i = 0; i < num_pes; i++) {
            if (pes[i] == global_me) {
                team_me = i;
                break;
            }
        }
    }
    struct fi_rma_iov *sub_alloc_buf = rofi->sub_alloc_buf;
    sub_alloc_buf[global_me].addr = (uint64_t)mr->start;
    sub_alloc_buf[global_me].key = fi_mr_key(mr->fid);
    DEBUG_MSG("Placing mr info (key: 0x%lx, addr: 0x%lx)... at local address: %p", sub_alloc_buf[global_me].key, sub_alloc_buf[global_me].addr, &sub_alloc_buf[global_me]);
    uint64_t sub_alloc_barrier_id = 0;
    rofi_transport_inner_barrier(rofi, &sub_alloc_barrier_id, rofi->sub_alloc_barrier_buf, pes, team_me, num_pes);

    for (int pe = team_me + 1; pe < num_pes; pe++) {
        uint64_t global_pe = pes[pe];
        void *src = (void *)&sub_alloc_buf[global_pe]; // this will be translated to the remote PE
        void *dst = src;                               // this will be our local data

        rofi_get_internal(dst, src, sizeof(struct fi_rma_iov), global_pe, 0);
    }
    for (int pe = 0; pe < team_me; pe++) {
        uint64_t global_pe = pes[pe];
        void *src = (void *)&sub_alloc_buf[global_pe]; // this will be translated to the remote PE
        void *dst = src;                               // this will be our local data

        rofi_get_internal(dst, src, sizeof(struct fi_rma_iov), global_pe, 0);
    }
    if (rofi_transport_get_wait_all(rofi)) {
        ERR_MSG("\t Error waiting for get");
    }
    for (int pe = 0; pe < num_pes; pe++) {
        uint64_t global_pe = pes[pe];
        mr->iov[global_pe] = sub_alloc_buf[global_pe];
        DEBUG_MSG("i: %d(pe: %d), addr: 0x%lx, key: 0x%lx  ", pe, global_pe, sub_alloc_buf[global_pe].addr, sub_alloc_buf[global_pe].key);
    }
    rofi_transport_inner_barrier(rofi, &sub_alloc_barrier_id, rofi->sub_alloc_barrier_buf, pes, team_me, num_pes);

    return 0;
}

int rofi_transport_sub_exchange_mr_info(rofi_transport_t *rofi, rofi_mr_desc *mr, uint64_t *pes, uint64_t num_pes) {
    if (rofi->desc.nodes == 1) {
        return 0;
    }
    if (!rofi->fi_collective) {
        return rofi_transport_sub_exchange_mr_info_manual(rofi, mr, pes, num_pes);
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

    // __OFI_PFOV_CXI__
    // NOTE: Currently CXI does not provide allgather as an OFI collective, so this code path is not followed
    // However, if it were, it would error because addresses are still virtual and need to be converted to offsets
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
}

int euclid_rem(int a, int b) {
    int r = a % b;
    return r >= 0 ? r : r + abs(b);
}

// This can support sub barriers
int rofi_transport_inner_barrier(rofi_transport_t *rofi, uint64_t *barrier_id, uint64_t *barrier_buf, uint64_t *pes, uint64_t me, uint64_t num_pes) {
    int n = 2;

    int num_rounds = ceil(log2((double)num_pes) / log2((double)n));

    int ret = 0;

    *barrier_id += 1;
    void *src = (void *)barrier_id;
    pthread_mutex_lock(&rofi->lock);
    ret = rofi_transport_progress(rofi);
    pthread_mutex_unlock(&rofi->lock);
    for (int round = 0; round < num_rounds; round++) {
        for (int i = 1; i <= n; i++) {
            int send_pe = euclid_rem((int)(me + i * pow(n + 1, round)), num_pes);
            send_pe = pes == NULL ? send_pe : pes[send_pe]; // if pes not null we are doing sub barrier

            // we need to store in the absolute pe location to prevent races.
            // allocations on multiple teams including the same PE can occur simultaneously,
            // the upper level runtime must ensure a given PE is only participating in one allocation at a time
            void *dst = (void *)(&barrier_buf[rofi->desc.nid]);

            DEBUG_MSG("%d Sending %d to %d %p - %p + %p", me, *barrier_id, send_pe, dst, rofi->mr->start, rofi->mr->iov[send_pe].addr);
            struct fi_rma_iov rma_iov;
#ifdef __OFI_PROV_CXI__
            // CXI uses offsets, so turn addr into offset
            rma_iov.addr = (uint64_t)(dst - rofi->mr->start + rofi->mr->iov[send_pe].addr) - (uint64_t)rofi->mr->start;
#else
            rma_iov.addr = (uint64_t)(dst - rofi->mr->start + rofi->mr->iov[send_pe].addr);
#endif
            rma_iov.key = rofi->mr->iov[send_pe].key;
            DEBUG_MSG("%d Sending barrier_id %lu to PE %d at remote addr 0x%lx with key 0x%lx", me, *barrier_id, send_pe, rma_iov.addr, rma_iov.key);
            ret = rofi_transport_put(rofi, &rma_iov, send_pe, src, sizeof(uint64_t), rofi->mr->mr_desc, NULL);
            if (ret) {
                return ret;
            }
        }
        for (int i = 1; i <= n; i++) {
            int recv_pe = euclid_rem((int)(me - i * pow(n + 1, round)), num_pes);
            recv_pe = pes == NULL ? recv_pe : pes[recv_pe]; // if pes not null we are doing sub barrier
            DEBUG_MSG("%d Receiving barrier_id %lu from PE %d at remote addr 0x%lx with key 0x%lx", me, *barrier_id, recv_pe, rofi->mr->iov[recv_pe].addr, rofi->mr->iov[recv_pe].key);

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

#ifdef __OFI_PROV_CXI__
// This and the following function are to avoid a race condition using RMA for the very first 
// barrier at the end of initialization
int rofi_transport_wait_on_cq(struct fid_cq *cq, struct fi_cq_entry *cqe, const int num_entries) {
  int ret;
  int count = 0;

  while (count < num_entries) {
    do {
      ret = fi_cq_read(cq, cqe, 1);
    } while (ret == -FI_EAGAIN);

    if (ret != 1) {
      ROFI_TRANSPORT_ERR_MSG("fi_cq_read", ret);
      struct fi_cq_err_entry ebuf = {0};
      int ret = fi_cq_readerr(cq, (void *)&ebuf, 0);
      if (ret > 0) {
        const char *errmsg = fi_cq_strerror(cq, ebuf.prov_errno, ebuf.err_data, NULL, 0);
        ERR_MSG("Error: %s\n", errmsg);
        abort();
        return ret;
      }
    }
    count++;
  }
  return 0;
}

int rofi_transport_barrier_msg(rofi_transport_t *rofi)
{
  int ret;

  unsigned int my_rank = rofi->desc.nid;
  unsigned int num_ranks = rofi->desc.nodes;
  struct fi_cq_entry cqe = {};
  uint8_t barrier_send_buf[1]; // buffer used for sends in the barrier
  uint8_t barrier_recv_buf[1]; // ditto but for recvs

  struct iovec iov_send = {barrier_send_buf, sizeof(uint8_t)};
  struct fi_msg msg_send = {};
  msg_send.msg_iov = &iov_send;
  msg_send.iov_count = 1;

  struct iovec iov_recv = {barrier_recv_buf, sizeof(uint8_t)};
  struct fi_msg msg_recv = {};
  msg_recv.msg_iov = &iov_recv;
  msg_recv.iov_count = 1;

  int parent = floor((my_rank-1)/2);
  int left_child = (2 * my_rank) + 1;
  int right_child = 2 * (my_rank + 1);

  int num_waits = 0;

  if (right_child < num_ranks) {
    num_waits++;
    msg_recv.addr = (rofi->remote_addrs)[right_child];
    ret = fi_recvmsg(rofi->ep, &msg_recv, FI_COMPLETION);
  }
  if (left_child < num_ranks) {
    num_waits++;
    msg_recv.addr = (rofi->remote_addrs)[left_child];
    ret = fi_recvmsg(rofi->ep, &msg_recv, FI_COMPLETION);
  }
  if (num_waits > 0) {
    ret = rofi_transport_wait_on_cq(rofi->cq, &cqe, num_waits);
  }
  if (my_rank > 0) {
    msg_send.addr = (rofi->remote_addrs)[parent];
    ret = fi_sendmsg(rofi->ep, &msg_send, 0);
  }

  if (my_rank > 0) {
    msg_recv.addr = rofi->remote_addrs[parent];
    ret = fi_recvmsg(rofi->ep, &msg_recv, FI_COMPLETION);
    ret = rofi_transport_wait_on_cq(rofi->cq, &cqe, 1);
  }
  if (left_child < num_ranks) {
    msg_send.addr = (rofi->remote_addrs)[left_child];
    ret = fi_sendmsg(rofi->ep, &msg_send, 0);
  }
  if (right_child < num_ranks) {
    msg_send.addr = (rofi->remote_addrs)[right_child];
    ret = fi_sendmsg(rofi->ep, &msg_send, 0);
  }
}

#endif // __OFI_PROV_CXI__
