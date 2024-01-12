/*
 * Copyright (c) 2013-2017 Intel Corporation.  All rights reserved.
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

#ifndef _TRANSPORT_H_
#define _TRANSPORT_H_

#if HAVE_CONFIG_H
#  include <config.h>
#endif /* HAVE_CONFIG_H */

#include <stdlib.h>
#include <inttypes.h>
#include <netinet/tcp.h>
#include <sys/uio.h>
#include <stdbool.h>

#include <rdma/fabric.h>
#include <rdma/fi_rma.h>
#include <rdma/fi_domain.h>

#define ROFI_TRANSPORT_ERR_MSG(call,retv) \
    do { fprintf(stderr, "[PE %d][ROFI TRANSPORT ERR][%s:%d] " call " failed: %s (%d)\n", \
           rt_get_rank(),__FILE__,__LINE__, fi_strerror(retv),(int)(retv)); } while(0)

 #define MIN(a,b) \
   ({ __typeof__ (a) _a = (a); \
       __typeof__ (b) _b = (b); \
     _a < _b ? _a : _b; })

int rofi_transport_fini(rofi_transport_t *rofi);
int rofi_transport_init(struct fi_info *hints,  rofi_transport_t *rofi);
int rofi_transport_init_fabric_resources( rofi_transport_t *rofi);
int rofi_transport_init_endpoint_resources( rofi_transport_t *rofi);
int rofi_transport_init_av(rofi_transport_t *rofi);


// void rofi_transport_progress(rofi_transport_t *rofi);
void rofi_transport_ctx_check_err(rofi_transport_t *rofi, int err);
void rofi_transport_check_rma_err(rofi_transport_t *rofi, int ret);

void rofi_transport_put_inject(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, const void *src_addr, size_t len);
void rofi_transport_put_large(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, const void *src_addr, size_t len, void *desc, void *context);
void rofi_transport_put(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, const void *src_addr, size_t len, void *desc, void *context);
void rofi_transport_put_wait_all(rofi_transport_t *rofi);

void rofi_transport_get_small(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, void *dst_addr, size_t len, void *desc, void *context);
void rofi_transport_get_large(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, void *dst_addr, size_t len, void *desc, void *context);
void rofi_transport_get(rofi_transport_t *rofi, struct fi_rma_iov *rma_iov, uint64_t pe, void *dst_addr, size_t len, void *desc, void *context);
void rofi_transport_get_wait_all(rofi_transport_t *rofi);


int rofi_transport_exchange_mr_info(rofi_transport_t *rofi, rofi_mr_desc *mr);

#endif /* _TRANSPORT_H_ */
