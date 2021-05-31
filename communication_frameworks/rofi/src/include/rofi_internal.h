#ifndef ROFI_INTERNAL_H
#define ROFI_INTERNAL_H

#include <rdma/fabric.h>
#include <rdma/fi_domain.h>
#include <rdma/fi_endpoint.h>
#include <rdma/fi_cm.h>

#include <uthash.h>

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

enum provider{shm, verbs};

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
	unsigned long          PageSize;
	unsigned long          tx_cntr;
	unsigned long          rx_cntr;
	enum provider          prov;
	
} rofi_desc_t;

typedef struct{
	void*              start;
	size_t             size;
	unsigned int       mode;
	struct fid_mr*     fid;
	struct fi_context2 ctx;
	uint64_t           mr_key;
	void*              mr_desc;
	struct fi_rma_iov* iov;
	UT_hash_handle     hh;
} rofi_mr_desc;

typedef struct{
	unsigned long       txid;
	struct fi_context2* ctx;
	UT_hash_handle      hh;
} rofi_ctx_desc;

extern rofi_desc_t rdesc;

int          rofi_init_internal(char*);
int          rofi_finit_internal(void);
unsigned int rofi_get_size_internal(void);
unsigned int rofi_get_id_internal(void);
int          rofi_put_internal(void*, void*, size_t, unsigned int, unsigned long);
int          rofi_get_internal(void*, void*, size_t, unsigned int, unsigned long);
int          rofi_send_internal(unsigned long, void*, size_t, unsigned long);
int          rofi_recv_internal(unsigned long, void*, size_t, unsigned long);
void*        rofi_alloc_internal(size_t, unsigned long);
int          rofi_release_internal(void*);
int          rofi_wait_internal(void);
void*        rofi_get_remote_addr_internal(void*, unsigned int);
void*        rofi_get_local_addr_from_remote_addr_internal(void*, unsigned int);

int rt_init(void);
int rt_finit(void);
int rt_get_rank(void);
int rt_get_size(void);
int rt_put(char*, void*, size_t);
int rt_get(int, char*, void*, size_t);
int rt_exchange(void);
void rt_barrier(void);

int            mr_init(void);
rofi_mr_desc*  mr_add(size_t, unsigned long);
rofi_mr_desc*  mr_get(const void*);
rofi_mr_desc*  mr_get_from_remote(const void* remote_addr, unsigned long remote_id);
int            mr_free(void);
int            mr_rm(void*);

int                 ctx_init(void);
unsigned long       ctx_new(void);
struct fi_context2* ctx_get(const unsigned long);
int                 ctx_rm(unsigned long);
int                 ctx_cleanup(void);
int                 ctx_free(void);
int                 ctx_get_lock(void);
int                 ctx_release_lock(void);

#endif
