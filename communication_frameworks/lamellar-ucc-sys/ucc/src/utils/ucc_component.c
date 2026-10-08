/**
 * Copyright (c) 2020-2024, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * See file LICENSE for terms.
 */
#include "config.h"
#include "ucc_malloc.h"
#include "ucc_component.h"
#include "ucc_log.h"
#include "ucc_math.h"
#include "core/ucc_global_opts.h"
#include "utils/ucc_string.h"

#include <sys/types.h>
#include <sys/stat.h>
#include <err.h>
#include <link.h>
#include <dlfcn.h>
#include <glob.h>
#include <unistd.h>
#include <assert.h>
#include <string.h>

#define IFACE_NAME_LEN_MAX                                                     \
    (UCC_MAX_FRAMEWORK_NAME_LEN + UCC_MAX_COMPONENT_NAME_LEN + 32)

static ucc_status_t ucc_component_load_one(const char *so_path,
                                           const char *framework_name,
                                           ucc_component_iface_t **c_iface)
{
    char                   framework_pattern[UCC_MAX_FRAMEWORK_NAME_LEN + 16];
    char                  *error, iface_struct[IFACE_NAME_LEN_MAX];
    void                  *handle;
    ucc_component_iface_t *iface;
    ptrdiff_t              basename_start;
    size_t                 iface_struct_name_len;

    ucc_snprintf_safe(framework_pattern, sizeof(framework_pattern), "ucc_%s_",
                      framework_name);
    basename_start =
        ((ptrdiff_t)ucc_strstr_last(so_path, framework_pattern) -
         (ptrdiff_t)so_path);
    if (basename_start < 0) {
        return UCC_ERR_NO_MESSAGE;
    }
    /* The name of the iface stract matches the basename of .so component
       object. basename_start - the starting position of the component name
       in the full .so path. The name_len is also decreased by 3 to remove
       ".so" extension from the name;
     */
    iface_struct_name_len = strlen(so_path) - basename_start - 3;
    ucc_strncpy_safe(iface_struct, so_path + basename_start,
                     iface_struct_name_len + 1);

    handle = dlopen(so_path, RTLD_LAZY);
    if (!handle) {
        error = dlerror();
        ucc_debug("failed to load UCC component library: %s (%s)",
                  so_path, error);
        goto error;
    }
    iface = (ucc_component_iface_t *)dlsym(handle, iface_struct);
    if ((error = dlerror()) != NULL) {
        ucc_error("failed to get iface %s from %s object (%s)", iface_struct,
                  so_path, error);
        goto iface_error;
    }
    if (!iface) {
        ucc_error("iface %s is NULL in %s object", iface_struct, so_path);
        goto iface_error;
    }
    iface->dl_handle = handle;
    iface->id        = ucc_str_hash_djb2(iface->name);
    (*c_iface)       = iface;
    return UCC_OK;

iface_error:
    dlclose(handle);
error:
    *c_iface = NULL;
    return UCC_ERR_NO_MESSAGE;
}

/*
 * Lamellar patch (not upstream UCC): built-in components for static builds.
 *
 * Upstream only discovers components by globbing
 * <component_path>/libucc_<framework>_*.so and dlopen()ing each one, so a
 * static-only build (libucc.a + lib/ucc/libucc_*.a linked into the final
 * executable) finds no CL/MC and ucc_init() fails.  Each component exposes
 * its iface as a global named ucc_<framework>_<name> (the same symbol the
 * dlopen path dlsym()s), so reference those weakly here: a component whose
 * archive is linked in (lamellar-ucc-sys links them with +whole-archive)
 * resolves to its iface, an absent one resolves to NULL.  The .so path is
 * kept for dynamic builds; a .so and a built-in with the same name are
 * de-duplicated, with the built-in winning.
 */
#define UCC_BUILTIN_COMPONENT(_fw, _name)                                      \
    extern ucc_component_iface_t ucc_##_fw##_##_name __attribute__((weak));
UCC_BUILTIN_COMPONENT(cl, basic)
UCC_BUILTIN_COMPONENT(cl, hier)
UCC_BUILTIN_COMPONENT(cl, doca_urom)
UCC_BUILTIN_COMPONENT(tl, self)
UCC_BUILTIN_COMPONENT(tl, ucp)
UCC_BUILTIN_COMPONENT(tl, cuda)
UCC_BUILTIN_COMPONENT(tl, mlx5)
UCC_BUILTIN_COMPONENT(tl, nccl)
UCC_BUILTIN_COMPONENT(tl, rccl)
UCC_BUILTIN_COMPONENT(tl, sharp)
UCC_BUILTIN_COMPONENT(mc, cpu)
UCC_BUILTIN_COMPONENT(mc, cuda)
UCC_BUILTIN_COMPONENT(mc, rocm)
UCC_BUILTIN_COMPONENT(ec, cpu)
UCC_BUILTIN_COMPONENT(ec, cuda)
UCC_BUILTIN_COMPONENT(ec, rocm)
UCC_BUILTIN_COMPONENT(sysinfo, ib)
UCC_BUILTIN_COMPONENT(sysinfo, cuda)
#undef UCC_BUILTIN_COMPONENT

#define UCC_BUILTIN_ENTRY(_fw, _name) {#_fw, &ucc_##_fw##_##_name},
static const struct {
    const char            *framework;
    ucc_component_iface_t *iface;
} ucc_builtin_components[] = {
    UCC_BUILTIN_ENTRY(cl, basic)
    UCC_BUILTIN_ENTRY(cl, hier)
    UCC_BUILTIN_ENTRY(cl, doca_urom)
    UCC_BUILTIN_ENTRY(tl, self)
    UCC_BUILTIN_ENTRY(tl, ucp)
    UCC_BUILTIN_ENTRY(tl, cuda)
    UCC_BUILTIN_ENTRY(tl, mlx5)
    UCC_BUILTIN_ENTRY(tl, nccl)
    UCC_BUILTIN_ENTRY(tl, rccl)
    UCC_BUILTIN_ENTRY(tl, sharp)
    UCC_BUILTIN_ENTRY(mc, cpu)
    UCC_BUILTIN_ENTRY(mc, cuda)
    UCC_BUILTIN_ENTRY(mc, rocm)
    UCC_BUILTIN_ENTRY(ec, cpu)
    UCC_BUILTIN_ENTRY(ec, cuda)
    UCC_BUILTIN_ENTRY(ec, rocm)
    UCC_BUILTIN_ENTRY(sysinfo, ib)
    UCC_BUILTIN_ENTRY(sysinfo, cuda)
};
#undef UCC_BUILTIN_ENTRY

#define UCC_N_BUILTIN_COMPONENTS                                               \
    (sizeof(ucc_builtin_components) / sizeof(ucc_builtin_components[0]))

/* Append the linked-in built-ins of framework_name to ifaces (capacity
 * UCC_N_BUILTIN_COMPONENTS past n_loaded), replacing a dlopen()ed component
 * of the same name.  Returns the new count. */
static int ucc_components_add_builtin(const char *framework_name,
                                      ucc_component_iface_t **ifaces,
                                      int n_loaded)
{
    ucc_component_iface_t *iface;
    int                    i, j;

    for (i = 0; i < UCC_N_BUILTIN_COMPONENTS; i++) {
        iface = ucc_builtin_components[i].iface;
        if (!iface || strcmp(ucc_builtin_components[i].framework,
                             framework_name)) {
            continue;
        }
        iface->dl_handle = NULL;
        iface->id        = ucc_str_hash_djb2(iface->name);
        for (j = 0; j < n_loaded; j++) {
            if (!strcmp(ifaces[j]->name, iface->name)) {
                break;
            }
        }
        if (j < n_loaded) {
            ucc_debug("using built-in %s component %s instead of .so",
                      framework_name, iface->name);
            ifaces[j] = iface;
        } else {
            ifaces[n_loaded++] = iface;
        }
    }
    return n_loaded;
}

#define CHECK_COMPONENT_UNIQ(_framework, _field)                               \
    do {                                                                       \
        ucc_component_iface_t **c = _framework->components;                    \
        int                     i, j;                                          \
        for (i = 0; i < framework->n_components; i++) {                        \
            for (j = i + 1; j < framework->n_components; j++) {                \
                if (c[i]->_field == c[j]->_field) {                            \
                    ucc_error("components %s and %s have the same "            \
                              "default " UCC_PP_MAKE_STRING(_field) " %lu",    \
                              c[i]->name, c[j]->name,                          \
                              (unsigned long)c[i]->_field);                    \
                    return UCC_ERR_INVALID_PARAM;                              \
                }                                                              \
            }                                                                  \
        }                                                                      \
    } while (0)

ucc_status_t
ucc_component_check_scores_uniq(ucc_component_framework_t *framework)
{
    CHECK_COMPONENT_UNIQ(framework, score);
    return UCC_OK;
}

static ucc_status_t
ucc_component_check_ids_uniq(ucc_component_framework_t *framework)
{
    CHECK_COMPONENT_UNIQ(framework, id);
    return UCC_OK;
}

ucc_status_t ucc_components_load(const char *framework_name,
                                 ucc_component_framework_t *framework)
{
    glob_t globbuf;
    int    i, n_loaded;
    char  *full_pattern;
    ucc_status_t            status;
    size_t                  pattern_size;
    ucc_component_iface_t **ifaces = NULL;

    framework->n_components = 0;
    framework->components   = NULL;

    if (strlen(framework_name) == 0 ||
        strlen(framework_name) > UCC_MAX_FRAMEWORK_NAME_LEN) {
        ucc_error("unsupported framework_name length: %s, len %zd",
                  framework_name, strlen(framework_name));
        return UCC_ERR_INVALID_PARAM;
    }

    /* Lamellar patch: in a static build, component constructors (e.g.
     * tl_ucp_iface_init loading its "tlcp_ucp" plugins) run at program start,
     * before ucc_init() has set component_path.  Upstream only reaches here
     * from a dlopen()ed component after init.  Skip the .so glob and use only
     * the built-ins in that case. */
    memset(&globbuf, 0, sizeof(globbuf));
    if (ucc_global_config.component_path) {
        pattern_size = strlen(ucc_global_config.component_path) +
                       strlen(framework_name) + 16;
        full_pattern = (char *)ucc_malloc(pattern_size, "full_pattern");
        if (!full_pattern) {
            ucc_error("failed to allocate %zd bytes for full_pattern",
                      pattern_size);
            return UCC_ERR_NO_MEMORY;
        }
        ucc_snprintf_safe(full_pattern, pattern_size, "%s/libucc_%s_*.so",
                          ucc_global_config.component_path, framework_name);
        glob(full_pattern, 0, NULL, &globbuf);
        ucc_free(full_pattern);
    }
    n_loaded          = 0;

    dlerror(); /* Clear any existing error */
    ifaces = (ucc_component_iface_t **)ucc_malloc(
        (globbuf.gl_pathc + UCC_N_BUILTIN_COMPONENTS) *
            sizeof(ucc_component_iface_t *), "ifaces");
    if (!ifaces) {
        ucc_error("failed to allocate %zd bytes for ifaces",
                  (globbuf.gl_pathc + UCC_N_BUILTIN_COMPONENTS) *
                      sizeof(ucc_component_iface_t *));
        return UCC_ERR_NO_MEMORY;
    }

    for (i = 0; i < globbuf.gl_pathc; i++) {
        status = ucc_component_load_one(globbuf.gl_pathv[i], framework_name,
                                        &ifaces[n_loaded]);
        if (status != UCC_OK) {
            continue;
        }
        n_loaded++;
    }

    assert(n_loaded <= globbuf.gl_pathc);
    n_loaded = ucc_components_add_builtin(framework_name, ifaces, n_loaded);
    if (globbuf.gl_pathc > 0) {
        globfree(&globbuf);
    }
    if (!n_loaded) {
        if (ifaces) {
            ucc_free(ifaces);
        }
        return UCC_ERR_NOT_FOUND;
    }

    ifaces = ucc_realloc(ifaces, n_loaded * sizeof(ucc_component_iface_t *),
                         "ifaces");
    framework->components   = ifaces;
    framework->n_components = n_loaded;
    if (UCC_OK != ucc_component_check_ids_uniq(framework)) {
        /* This can only happen when the new component is added
           (potentially as a plugin - black box) and its name hash
           will produce collision with some other component.
           This has nearly 0 probability and must be resolved by
           the component developer via just a rename of a component. */
        ucc_error("all components of a framwork must have uniq name hash");
        status = UCC_ERR_INVALID_PARAM;
        goto err;
    }

    framework->names.names =
        ucc_malloc(sizeof(char *) * n_loaded, "components_names");
    if (!framework->names.names) {
        ucc_error("failed to allocate %zd bytes for components names",
                  sizeof(char *) * n_loaded);
        status = UCC_ERR_NO_MEMORY;
        goto err;
    }
    framework->names.count = n_loaded;
    for (i = 0; i < n_loaded; i++) {
        framework->names.names[i] = strdup(framework->components[i]->name);
    }

    /* Lamellar patch (not upstream): same class of check as UCX's
     * ucp_warn_missing_ib_component() -- a static build only gets a given
     * component's *other* object files (the ones implementing its actual
     * collective ops, not just the weak iface-struct definition referenced
     * above) if that component's archive was linked with
     * -Wl,--whole-archive. A consumer linking this vendored UCC directly
     * and forgetting lib/ucc/libucc_tl_ucp.a's whole-archive treatment
     * doesn't hit an error here -- the "tl" framework still loads
     * successfully via tl_self alone -- it just silently never gets a
     * UCX-backed TL. dlsym() for ucp_init as a proxy for "UCX was actually
     * linked into this binary at all"; if it's present but tl_ucp's weak
     * iface never resolved, the omission is on UCC's own whole-archive
     * list, not on UCX being unavailable. */
    ucc_component_iface_t *ucp_iface = &ucc_tl_ucp;
    if (!strcmp(framework_name, "tl") && !ucp_iface &&
        dlsym(RTLD_DEFAULT, "ucp_init") != NULL) {
        ucc_warn("UCX (ucp_init) is linked into this process, but UCC's "
                 "own \"tl_ucp\" component never registered -- collectives "
                 "will run without a UCX-backed transport layer. If this "
                 "is a static build, the consumer likely linked "
                 "libucc_tl_ucp.a without -Wl,--whole-archive (see "
                 "lamellar-ucc-sys's build.rs / NATIVE_DEPENDENCIES.md for "
                 "the required pattern).");
    }

    return UCC_OK;
err:
    ucc_free(framework->components);
    return status;
}

ucc_component_iface_t* ucc_get_component(ucc_component_framework_t *framework,
                                         const char *component_name)
{
    int i;
    for (i = 0; i < framework->n_components; i++) {
        if (0 == strcmp(framework->components[i]->name, component_name)) {
            return framework->components[i];
        }
    }
    return NULL;
}

char* ucc_get_framework_components_list(ucc_component_framework_t *framework,
                                        const char* delimiter)
{
    char  *list = NULL;
    size_t len  = 0;
    int    i;

    for (i = 0; i < framework->n_components; i++) {
        /* component name + ',' delimiter */
        len += strlen(framework->components[i]->name) + 1;
    }
    if (len) {
        list = ucc_malloc(len + 1, "components_list");
        if (!list) {
            ucc_error("failed to allocate %zd bytes for components_list", len);
            return NULL;
        }
        list[0] = '\0';
        for (i = 0; i < framework->n_components; i++) {
            strncat(list, framework->components[i]->name, len);
            len -= strlen(framework->components[i]->name);
            if (i < framework->n_components - 1) {
                strncat(list, delimiter, len);
                len -= strlen(delimiter);
            }
        }
    }
    return list;
}
