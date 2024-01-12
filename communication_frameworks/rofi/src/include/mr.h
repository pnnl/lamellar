
#ifndef ROFI_MR_H
#define ROFI_MR_H



typedef struct {
    void *start;
    size_t size;
    unsigned int mode;
    struct fid_mr *fid;
    struct fi_context2 ctx;
    uint64_t mr_key;
    void *mr_desc;
    struct fi_rma_iov *iov;
    UT_hash_handle hh;
} rofi_mr_desc; 

int mr_init(void);
rofi_mr_desc *mr_add(rofi_transport_t *rofi, size_t, unsigned long);
rofi_mr_desc *mr_get(const void *);
rofi_mr_desc *mr_get_from_remote(const void *, unsigned long);
int mr_free(void);
int mr_rm(void *);

#endif