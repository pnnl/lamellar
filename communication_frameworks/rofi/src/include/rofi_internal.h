#ifndef ROFI_INTERNAL_H
#define ROFI_INTERNAL_H

#include <rdma/fabric.h>
#include <rdma/fi_domain.h>
#include <rdma/fi_endpoint.h>
#include <rdma/fi_cm.h>

#define ROFI_STATUS_NONE   0
#define ROFI_STATUS_START  1
#define ROFI_STATUS_ACTIVE 2
#define ROFI_STATUS_TERM   3
#define ROFI_STATUS_END    4
#define ROFI_STATUS_ERR    5

#define ROFI_SERVER        1
#define ROFI_CLIENT        2

#define MAX_HOSTNAME_LEN 256
#define KILO 1024UL
#define MEGA 1024UL*KILO
#define GIGA 1024UL*MEGA

#define ROFI_HEAP_NOTALLOCATED 0
#define ROFI_HEAP_ALLOCATED    1
#define ROFI_HEAP_REGISTERED   2
#define ROFI_HEAP_INVALID      3

#define ROFI_ASYNC             0x1
#define ROFI_SYNC              0x2

typedef struct ofi_status_struct{
	struct fi_info*        hints;
	struct fi_info*        info;
	
	struct fi_domain_attr* dom_attr;
	
	struct fid_fabric*     ffid;
	struct fid_domain*     dfid;
	struct fid_av*         avfid;
	struct fid_eq*         eqfid;
	struct fid_ep*         epfid;
	struct fid_cq*         cqfid;
} ofi_status_t;

typedef struct ofi_ctx_struct {
    int                             id;
    long                            options;
    struct fid_ep*                  ep;
    struct fid_cntr*                put_cntr;
    struct fid_cntr*                get_cntr;
    struct fid_cq*                  cq;
} ofi_ctx_t;

typedef struct rofi_desc_struct{
	unsigned long          status;
	unsigned int           nodes;
	unsigned int           nid;
	int                    addrlen;
} rofi_desc_t;

extern rofi_desc_t rdesc;

int          rofi_init_internal(void);
int          rofi_finit_internal(void);
unsigned int rofi_get_size_internal(void);
unsigned int rofi_get_id_internal(void);
int          rofi_put_internal(void*, const void*, size_t, unsigned int, unsigned long);
int          rofi_get_internal(void*, const void*, size_t, unsigned int, unsigned long);
void*        rofi_alloc_internal(size_t, unsigned long);
int          rofi_release_internal(void);
int          rofi_wait_internal(void);

int rt_init(void);
int rt_finit(void);
int rt_get_rank(void);
int rt_get_size(void);
int rt_put(char*, void*, size_t);
int rt_get(int, char*, void*, size_t);
int rt_exchange(void);
void rt_barrier(void);
#endif
