#ifndef ROFI_RT_H
#define ROFI_RT_H

int rt_init(void);
int rt_finit(void);
int rt_get_rank(void);
int rt_get_size(void);
int rt_put(char *, void *, size_t);
int rt_get(int, char *, void *, size_t);
int rt_exchange(void);
int rt_exchange_data(char *data_name, void *data, size_t size, void *recvbuf, unsigned int rank, unsigned int nranks);
void rt_barrier(void);

#endif