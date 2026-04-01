#ifndef ROFI_H
#define ROFI_H
#include <stddef.h>
#include <stdint.h>

#define ROFI_ERR_ALLOC 0x01

typedef enum {
	ROFI_DATATYPE_INT8 = 0,
	ROFI_DATATYPE_UINT8,
	ROFI_DATATYPE_INT16,
	ROFI_DATATYPE_UINT16,
	ROFI_DATATYPE_INT32,
	ROFI_DATATYPE_UINT32,
	ROFI_DATATYPE_INT64,
	ROFI_DATATYPE_UINT64,
	ROFI_DATATYPE_FLOAT,
	ROFI_DATATYPE_DOUBLE,
	ROFI_DATATYPE_FLOAT_COMPLEX,
	ROFI_DATATYPE_DOUBLE_COMPLEX,
} rofi_datatype_t;

typedef enum {
	ROFI_ATOMIC_OP_MIN = 0,
	ROFI_ATOMIC_OP_MAX,
	ROFI_ATOMIC_OP_SUM,
	ROFI_ATOMIC_OP_PROD,
	ROFI_ATOMIC_OP_LOR,
	ROFI_ATOMIC_OP_LAND,
	ROFI_ATOMIC_OP_BOR,
	ROFI_ATOMIC_OP_BAND,
	ROFI_ATOMIC_OP_LXOR,
	ROFI_ATOMIC_OP_BXOR,
	ROFI_ATOMIC_OP_READ,
	ROFI_ATOMIC_OP_CSWAP,
	ROFI_ATOMIC_OP_CSWAP_NE,
	ROFI_ATOMIC_OP_CSWAP_LE,
	ROFI_ATOMIC_OP_CSWAP_LT,
	ROFI_ATOMIC_OP_CSWAP_GE,
	ROFI_ATOMIC_OP_CSWAP_GT,
	ROFI_ATOMIC_OP_MSWAP,
	ROFI_ATOMIC_OP_WRITE,
} rofi_atomic_op_t;

int rofi_init(char *, char *);
int rofi_finit(void);
unsigned int rofi_get_size(void);
unsigned int rofi_get_id(void);
int rofi_flush(void);
int rofi_put(void *, void *, size_t, unsigned int, unsigned long);
int rofi_iput(void *, void *, size_t, unsigned int, unsigned long);
int rofi_get(void *, void *, size_t, unsigned int, unsigned long);
int rofi_iget(void *, void *, size_t, unsigned int, unsigned long);
int rofi_send(unsigned int, void *, size_t, unsigned long);
int rofi_recv(void *, size_t, unsigned long);
int rofi_alloc(size_t, unsigned long, void **);
int rofi_sub_alloc(size_t, unsigned long, void **, uint64_t *, uint64_t);
int rofi_release(void *);
int rofi_sub_release(void *, uint64_t *, uint64_t);
void rofi_barrier(void);
int rofi_wait(void);
void *rofi_get_remote_addr(void *, unsigned int);
void *rofi_get_local_addr_from_remote_addr(void *, unsigned int);
int rofi_has_atomics(void);
int rofi_query_atomic(rofi_datatype_t, rofi_atomic_op_t);
int rofi_query_fetch_atomic(rofi_datatype_t, rofi_atomic_op_t);
int rofi_query_compare_atomic(rofi_datatype_t, rofi_atomic_op_t);
ssize_t rofi_atomic_op(void *, const void *, size_t, rofi_datatype_t, rofi_atomic_op_t, unsigned int);
ssize_t rofi_atomic_fetch(void *, const void *, void *, size_t, rofi_datatype_t, rofi_atomic_op_t, unsigned int);
ssize_t rofi_compare_atomic(void *, const void *, const void *, void *, size_t, rofi_datatype_t, rofi_atomic_op_t, unsigned int);
#endif
