/* -*- Mode: C; c-basic-offset:4 ; indent-tabs-mode:nil ; -*- */

#include "config.h"

#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <pmix.h>

#include "pmi.h"

#ifndef PMIX_PMI_VALUE_LEN_MAX
#define PMIX_PMI_VALUE_LEN_MAX 65536
#endif

#ifndef PMIX_PMI_KEY_LEN_MAX
#ifdef PMIX_MAX_KEYLEN
#define PMIX_PMI_KEY_LEN_MAX (PMIX_MAX_KEYLEN + 1)
#else
#define PMIX_PMI_KEY_LEN_MAX 512
#endif
#endif

static int pmi_initialized;
static int pmi_rank;
static int pmi_size = 1;
static pmix_proc_t pmi_proc;
static char pmi_nspace[sizeof(((pmix_proc_t *) 0)->nspace)];

static int pmi_status_to_pmi(pmix_status_t status)
{
    return (status == PMIX_SUCCESS) ? PMI_SUCCESS : PMI_FAIL;
}

static void pmi_release_value(pmix_value_t *value)
{
#if 1
#endif
#if defined(PMIX_VALUE_RELEASE)
    PMIX_VALUE_RELEASE(value);
#else
    (void) value;
#endif
}

static void pmi_set_proc(pmix_proc_t *proc, pmix_rank_t rank)
{
    memset(proc, 0, sizeof(*proc));
    strncpy(proc->nspace, pmi_nspace, sizeof(proc->nspace) - 1);
    proc->nspace[sizeof(proc->nspace) - 1] = '\0';
    proc->rank = rank;
}

static int pmi_parse_rank_from_key(const char key[], pmix_rank_t *rank)
{
    const char *cursor;
    char *endptr;
    long parsed_rank;

    if (strncmp(key, "rofi-", 5) != 0) {
        return 0;
    }

    cursor = key + 5;
    if (!isdigit((unsigned char) *cursor)) {
        return 0;
    }

    parsed_rank = strtol(cursor, &endptr, 10);
    if (endptr == cursor || *endptr != '-') {
        return 0;
    }

    *rank = (pmix_rank_t) parsed_rank;
    return 1;
}

static int pmi_value_to_int(pmix_value_t *value, int *out)
{
    if (value == NULL || out == NULL) {
        return PMI_ERR_INVALID_ARG;
    }

    switch (value->type) {
    case PMIX_UINT32:
        *out = (int) value->data.uint32;
        return PMI_SUCCESS;
    case PMIX_INT:
        *out = value->data.integer;
        return PMI_SUCCESS;
    case PMIX_STRING:
        if (value->data.string == NULL) {
            return PMI_FAIL;
        }
        *out = atoi(value->data.string);
        return PMI_SUCCESS;
    default:
        return PMI_FAIL;
    }
}

static int pmi_load_job_size(void)
{
    pmix_proc_t wildcard;
    pmix_value_t *value;
    pmix_status_t status;
    int rc;

    value = NULL;
    pmi_set_proc(&wildcard, PMIX_RANK_WILDCARD);
    status = PMIx_Get(&wildcard, PMIX_JOB_SIZE, NULL, 0, &value);
    if (status != PMIX_SUCCESS) {
        return PMI_FAIL;
    }

    rc = pmi_value_to_int(value, &pmi_size);
    pmi_release_value(value);
    return rc;
}

int PMI_Init(int *spawned)
{
    pmix_status_t status;

    if (spawned == NULL) {
        return PMI_ERR_INVALID_ARG;
    }

    if (pmi_initialized) {
        *spawned = 0;
        return PMI_SUCCESS;
    }

    memset(&pmi_proc, 0, sizeof(pmi_proc));
    status = PMIx_Init(&pmi_proc, NULL, 0);
    if (status != PMIX_SUCCESS) {
        return PMI_FAIL;
    }

    pmi_rank = (int) pmi_proc.rank;
    strncpy(pmi_nspace, pmi_proc.nspace, sizeof(pmi_nspace) - 1);
    pmi_nspace[sizeof(pmi_nspace) - 1] = '\0';

    if (PMI_SUCCESS != pmi_load_job_size()) {
        (void) PMIx_Finalize(NULL, 0);
        memset(&pmi_proc, 0, sizeof(pmi_proc));
        pmi_nspace[0] = '\0';
        return PMI_FAIL;
    }

    pmi_initialized = 1;
    *spawned = 0;
    return PMI_SUCCESS;
}

int PMI_Initialized(int *initialized)
{
    if (initialized == NULL) {
        return PMI_ERR_INVALID_ARG;
    }

    *initialized = pmi_initialized ? 1 : 0;
    return PMI_SUCCESS;
}

int PMI_Finalize(void)
{
    if (!pmi_initialized) {
        return PMI_SUCCESS;
    }
    if (PMIX_SUCCESS != PMIx_Finalize(NULL, 0)) {
        return PMI_FAIL;
    }

    pmi_initialized = 0;
    pmi_rank = 0;
    pmi_size = 1;
    memset(&pmi_proc, 0, sizeof(pmi_proc));
    pmi_nspace[0] = '\0';
    return PMI_SUCCESS;
}

int PMI_Get_size(int *size)
{
    if (size == NULL) {
        return PMI_ERR_INVALID_ARG;
    }

    *size = pmi_size;
    return PMI_SUCCESS;
}

int PMI_Get_rank(int *rank)
{
    if (rank == NULL) {
        return PMI_ERR_INVALID_ARG;
    }

    *rank = pmi_rank;
    return PMI_SUCCESS;
}

int PMI_Get_universe_size(int *size)
{
    return PMI_Get_size(size);
}

int PMI_Get_appnum(int *appnum)
{
    char *value;

    if (appnum == NULL) {
        return PMI_ERR_INVALID_ARG;
    }

    value = getenv("PMIX_APPNUM");
    *appnum = (value == NULL) ? 0 : atoi(value);
    return PMI_SUCCESS;
}

int PMI_Publish_name(const char service_name[], const char port[])
{
    (void) service_name;
    (void) port;
    return PMI_FAIL;
}

int PMI_Unpublish_name(const char service_name[])
{
    (void) service_name;
    return PMI_FAIL;
}

int PMI_Lookup_name(const char service_name[], char port[])
{
    (void) service_name;
    (void) port;
    return PMI_FAIL;
}

int PMI_Barrier(void)
{
    pmix_info_t info;


    if (!pmi_initialized || pmi_size <= 1) {
        return PMI_SUCCESS;
    }

    memset(&info, 0, sizeof(info));
    strncpy(info.key, PMIX_COLLECT_DATA, sizeof(info.key) - 1);
    info.key[sizeof(info.key) - 1] = '\0';
    info.value.type = PMIX_BOOL;
    info.value.data.flag = 1;

    return pmi_status_to_pmi(PMIx_Fence(NULL, 0, &info, 1));
}

int PMI_Abort(int exit_code, const char error_msg[])
{
    if (pmi_initialized) {
        (void) PMIx_Abort(exit_code, error_msg, NULL, 0);
    }

    if (error_msg != NULL) {
    }

    abort();
}

int PMI_KVS_Get_my_name(char kvsname[], int length)
{
    size_t required;

    if (kvsname == NULL) {
        return PMI_ERR_INVALID_ARG;
    }

    required = strlen(pmi_nspace) + 1;
    if (length <= 0 || (size_t) length < required) {
        return PMI_ERR_INVALID_LENGTH;
    }

    memcpy(kvsname, pmi_nspace, required);
    return PMI_SUCCESS;
}

int PMI_KVS_Get_name_length_max(int *length)
{
    if (length == NULL) {
        return PMI_ERR_INVALID_ARG;
    }

    *length = (int) sizeof(pmi_nspace);
    return PMI_SUCCESS;
}

int PMI_KVS_Get_key_length_max(int *length)
{
    if (length == NULL) {
        return PMI_ERR_INVALID_ARG;
    }

    *length = PMIX_PMI_KEY_LEN_MAX;
    return PMI_SUCCESS;
}

int PMI_KVS_Get_value_length_max(int *length)
{
    if (length == NULL) {
        return PMI_ERR_INVALID_ARG;
    }

    *length = PMIX_PMI_VALUE_LEN_MAX;
    return PMI_SUCCESS;
}

int PMI_KVS_Put(const char kvsname[], const char key[], const char value[])
{
    pmix_value_t pmix_value;

    (void) kvsname;

    if (key == NULL || value == NULL) {
        return PMI_ERR_INVALID_ARG;
    }

    memset(&pmix_value, 0, sizeof(pmix_value));
    pmix_value.type = PMIX_STRING;
    pmix_value.data.string = (char *) value;

    return pmi_status_to_pmi(PMIx_Put(PMIX_GLOBAL, key, &pmix_value));
}

int PMI_KVS_Commit(const char kvsname[])
{
    (void) kvsname;

    if (!pmi_initialized || pmi_size <= 1) {
        return PMI_SUCCESS;
    }

    return pmi_status_to_pmi(PMIx_Commit());
}

int PMI_KVS_Get(const char kvsname[], const char key[], char value[], int length)
{
    pmix_proc_t target;
    pmix_rank_t key_rank;
    pmix_value_t *pmix_value;
    pmix_status_t status;

    (void) kvsname;

    if (key == NULL || value == NULL || length <= 0) {
        return PMI_ERR_INVALID_ARG;
    }

    if (pmi_parse_rank_from_key(key, &key_rank)) {
        pmi_set_proc(&target, key_rank);
    }
    else {
        pmi_set_proc(&target, PMIX_RANK_WILDCARD);
    }

    pmix_value = NULL;
    status = PMIx_Get(&target, key, NULL, 0, &pmix_value);
    if (status != PMIX_SUCCESS || pmix_value == NULL) {
        return PMI_FAIL;
    }

    if (pmix_value->type != PMIX_STRING || pmix_value->data.string == NULL) {
        pmi_release_value(pmix_value);
        return PMI_FAIL;
    }

    if ((int) strlen(pmix_value->data.string) + 1 > length) {
        pmi_release_value(pmix_value);
        return PMI_ERR_INVALID_LENGTH;
    }

    strncpy(value, pmix_value->data.string, (size_t) length - 1);
    value[length - 1] = '\0';
    pmi_release_value(pmix_value);
    return PMI_SUCCESS;
}

int PMI_Spawn_multiple(int count,
                       const char *cmds[],
                       const char **argvs[],
                       const int maxprocs[],
                       const int info_keyval_sizesp[],
                       const PMI_keyval_t *info_keyval_vectors[],
                       int preput_keyval_size,
                       const PMI_keyval_t preput_keyval_vector[],
                       int errors[])
{
    (void) count;
    (void) cmds;
    (void) argvs;
    (void) maxprocs;
    (void) info_keyval_sizesp;
    (void) info_keyval_vectors;
    (void) preput_keyval_size;
    (void) preput_keyval_vector;
    (void) errors;
    return PMI_FAIL;
}