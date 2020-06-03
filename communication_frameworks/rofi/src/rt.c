#include "config.h"

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <assert.h>
#include <unistd.h>

#include <pmi.h>
#include <uthash.h>

#include <rofi_debug.h>

static int rank = -1;
static int size = 0, node_size = 0;
static char *kvs_name, *kvs_key, *kvs_value;
static int max_name_len, max_key_len, max_val_len;
static int initialized_pmi = 0;
static int *location_array = NULL;

#define SINGLETON_KEY_LEN 128
#define SINGLETON_VAL_LEN 256
#define MAX_HOSTNAME_LEN  256

typedef struct {
    char key[SINGLETON_KEY_LEN];
    char val[SINGLETON_VAL_LEN];
    UT_hash_handle hh;
} singleton_kvs_t;

singleton_kvs_t *singleton_kvs = NULL;

int rt_get_rank(void)
{
    return rank;
}


int rt_get_size(void)
{
    return size;
}

int rt_init(void)
{
	int initialized;

	DEBUG_MSG("Initializing ROFI RT...");
	if (PMI_SUCCESS != PMI_Initialized(&initialized)) {
		return 1;
	}
	
	if (!initialized) {
		if (PMI_SUCCESS != PMI_Init(&initialized)) {
			return 2;
		}
		else {
			initialized_pmi = 1;
		}
	}
	
	if (PMI_SUCCESS != PMI_Get_rank(&rank)) {
		return 3;
	}
	
	if (PMI_SUCCESS != PMI_Get_size(&size)) {
		return 4;
	}
	
	if (size > 1) {
		if (PMI_SUCCESS != PMI_KVS_Get_name_length_max(&max_name_len)) {
			return 5;
		}
		kvs_name = (char*) malloc(max_name_len);
		if (NULL == kvs_name) return 6;
		
		if (PMI_SUCCESS != PMI_KVS_Get_key_length_max(&max_key_len)) {
			return 7;
		}
		if (PMI_SUCCESS != PMI_KVS_Get_value_length_max(&max_val_len)) {
			return 8;
		}
		if (PMI_SUCCESS != PMI_KVS_Get_my_name(kvs_name, max_name_len)) {
			return 9;
		}
		
		//        if (enable_node_ranks) {
		location_array = malloc(sizeof(int) * size);
		if (NULL == location_array) return 10;
		//       }
	}
	else {
		/* Use a local KVS for singleton runs */
		max_key_len = SINGLETON_KEY_LEN;
		max_val_len = SINGLETON_VAL_LEN;
		kvs_name = NULL;
		max_name_len = 0;
	}
	
	kvs_key = (char*) malloc(max_key_len);
	if (NULL == kvs_key) return 11;
	
	kvs_value = (char*) malloc(max_val_len);
	if (NULL == kvs_value) return 12;

	DEBUG_MSG("ROFI RT initialized (%d/%d)!", rt_get_rank(), rt_get_size());
	return 0;
}


int rt_finit(void)
{
	DEBUG_MSG("Shutting down ROTI RT...");
	if (location_array) {
		free(location_array);
	}
	
	if (initialized_pmi) {
		PMI_Finalize();
		initialized_pmi = 0;
	}
	DEBUG_MSG("ROTI RT finalized.");
	return 0;
}


void rt_abort(int exit_code, const char msg[])
{

#ifdef HAVE___BUILTIN_TRAP
    if (shmem_internal_params.TRAP_ON_ABORT)
        __builtin_trap();
#endif

    if (size == 1) {
        fprintf(stderr, "%s\n", msg);
        exit(exit_code);
    }

    PMI_Abort(exit_code, msg);

    /* PMI_Abort should not return */
    abort();
}



int rt_get_node_rank(int pe)
{
    assert(pe < size && pe >= 0);

    if (size == 1) {
        return 0;
    } else {
        return location_array[pe];
    }
}


int rt_get_node_size(void)
{
    if (size == 1) {
        return 1;
    } else {
        return node_size;
    }
}



void rt_barrier(void)
{
    PMI_Barrier();
}



int rt_util_encode(const void *inval, int invallen, char *outval,
                          int outvallen)
{
    static unsigned char encodings[] = {
        '0','1','2','3','4','5','6','7', \
        '8','9','a','b','c','d','e','f' };
    int i;

    if (invallen * 2 + 1 > outvallen) {
        return 1;
    }

    for (i = 0; i < invallen; i++) {
        outval[2 * i] = encodings[((unsigned char *)inval)[i] & 0xf];
        outval[2 * i + 1] = encodings[((unsigned char *)inval)[i] >> 4];
    }

    outval[invallen * 2] = '\0';

    return 0;
}

int rt_util_decode(const char *inval, void *outval, size_t outvallen)
{
    size_t i;
    char *ret = (char*) outval;

    if (outvallen != strlen(inval) / 2) {
        return 1;
    }

    for (i = 0 ; i < outvallen ; ++i) {
        if (*inval >= '0' && *inval <= '9') {
            ret[i] = *inval - '0';
        } else {
            ret[i] = *inval - 'a' + 10;
        }
        inval++;
        if (*inval >= '0' && *inval <= '9') {
            ret[i] |= ((*inval - '0') << 4);
        } else {
            ret[i] |= ((*inval - 'a' + 10) << 4);
        }
        inval++;
    }

    return 0;
}


int rt_put(char *key, void *value, size_t valuelen)
{
    snprintf(kvs_key, max_key_len, "rofi-%lu-%s", (long unsigned) rank, key);
    if (0 != rt_util_encode(value, valuelen, kvs_value,
                                       max_val_len)) {
        return 1;
    }

    if (size == 1) {
        singleton_kvs_t *e = malloc(sizeof(singleton_kvs_t));
        if (e == NULL) return 3;
        assert(max_key_len <= SINGLETON_KEY_LEN);
        assert(max_val_len <= SINGLETON_VAL_LEN);
        strncpy(e->key, kvs_key, max_key_len);
        strncpy(e->val, kvs_value, max_val_len);
        HASH_ADD_STR(singleton_kvs, key, e);
    } else {
        if (PMI_SUCCESS != PMI_KVS_Put(kvs_name, kvs_key, kvs_value)) {
            return 2;
        }
    }

    DEBUG_MSG("Added (%s,%zu)", key, valuelen);
    return 0;
}


int rt_get(int pe, char *key, void *value, size_t valuelen)
{
    snprintf(kvs_key, max_key_len, "rofi-%lu-%s", (long unsigned) pe, key);
    if (size == 1) {
        singleton_kvs_t *e;
        HASH_FIND_STR(singleton_kvs, kvs_key, e);
        if (e == NULL)
            return 3;
        kvs_value = e->val;
    }
    else {
        if (PMI_SUCCESS != PMI_KVS_Get(kvs_name, kvs_key, kvs_value, max_val_len)) {
            return 1;
        }
    }
    if (0 != rt_util_decode(kvs_value, value, valuelen)) {
        return 2;
    }

    return 0;
}

int rt_util_populate_node(int *location_array, int size, int *node_size)
{
    int ret, i, n_node_pes = 0;
    char hostname[MAX_HOSTNAME_LEN+1];

    ret = gethostname(hostname, MAX_HOSTNAME_LEN);
    if (ret != 0) {
        ERR_MSG("gethostname failed (%d)", ret);
        return ret;
    }

    /* gethostname() doesn't guarantee null-termination, add NIL */
    hostname[MAX_HOSTNAME_LEN] = '\0';

    for (i = 0; i < size; i++) {
        char peer_hostname[MAX_HOSTNAME_LEN+1];
        size_t hlen;

        ret = rt_get(i, "hostname_len", &hlen, sizeof(size_t));
        if (ret != 0) {
            ERR_MSG("Failed during hostname_len read from KVS (%d)", ret);
            return ret;
        }

        ret = rt_get(i, "hostname", peer_hostname, hlen);
        if (ret != 0) {
            ERR_MSG("Failed during hostname read from KVS (%d)", ret);
            return ret;
        }
        if (strncmp(hostname, peer_hostname, hlen) == 0) {
            location_array[i] = n_node_pes;
            n_node_pes++;
        }
        else
            location_array[i] = -1;
    }

    if (n_node_pes < 1 || n_node_pes > size) {
        ERR_MSG("Invalid node size (%d)\n", n_node_pes);
        return 1;
    }

    *node_size = n_node_pes;

    return 0;
}


/* Put the hostname into the runtime KVS */
int rt_util_put_hostname(void)
{
    char hostname[MAX_HOSTNAME_LEN+1];
    int ret;

    ret = gethostname(hostname, MAX_HOSTNAME_LEN);
    if (ret != 0) {
        ERR_MSG("gethostname failed (%d)", ret);
        return ret;
    }

    /* gethostname() doesn't guarantee null-termination, add NIL */
    hostname[MAX_HOSTNAME_LEN] = '\0';

    size_t hostname_len = strlen(hostname);

    ret = rt_put("hostname_len", &hostname_len, sizeof(size_t));
    if (ret != 0) {
        ERR_MSG("Failed during hostname_len store to KVS: (%d)", ret);
        return ret;
    }

    ret = rt_put("hostname", hostname, hostname_len);
    if (ret != 0) {
        ERR_MSG("Failed during hostname store to KVS: (%d)", ret);
        return ret;
    }

    return 0;
}


int rt_exchange(void)
{
    int ret;

    /* Use singleton KVS for single process jobs */
    if (size == 1)
        return 0;

    if (location_array) {
        ret = rt_util_put_hostname();
        if (ret != 0) {
            ERR_MSG("KVS hostname put (%d)", ret);
            return 4;
        }
    }

    if (PMI_SUCCESS != PMI_KVS_Commit(kvs_name)) {
        return 5;
    }

    if (PMI_SUCCESS != PMI_Barrier()) {
        return 6;
    }

    if (location_array) {
        ret = rt_util_populate_node(location_array, size, &node_size);
        if (0 != ret) {
            ERR_MSG("Node PE mapping failed (%d)\n", ret);
            return 7;
        }
    }

    return 0;
}

