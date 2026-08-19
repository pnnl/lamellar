#ifndef ROFI_CONTEXT_H
#define ROFI_CONTEXT_H

int ctx_init(void);
unsigned long ctx_new(void);
struct fi_context2 *ctx_get(const unsigned long);
int ctx_rm(unsigned long);
int ctx_cleanup(void);
int ctx_free(void);
int ctx_get_lock(void);
int ctx_release_lock(void);

#endif