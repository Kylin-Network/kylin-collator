/***********************************************************************
 * Copyright (c) 2013-2015 Pieter Wuille                               *
 * Distributed under the MIT software license, see the accompanying    *
 * file COPYING or https://www.opensource.org/licenses/mit-license.php.*
 ***********************************************************************/

#define SECP256K1_BUILD

#include "../include/secp256k1.h"
#include "../include/secp256k1_preallocated.h"

#include "assumptions.h"
#include "util.h"
#include "field_impl.h"
#include "scalar_impl.h"
#include "group_impl.h"
#include "ecmult_impl.h"
#include "ecmult_const_impl.h"
#include "ecmult_gen_impl.h"
#include "ecdsa_impl.h"
#include "eckey_impl.h"
#include "hash_impl.h"
#include "scratch_impl.h"
#include "selftest.h"

#ifdef SECP256K1_NO_BUILD
# error "secp256k1.h processed without SECP256K1_BUILD defined while building secp256k1.c"
#endif

#if defined(VALGRIND)
# include <valgrind/memcheck.h>
#endif

#define ARG_CHECK(cond) do { \
    if (EXPECT(!(cond), 0)) { \
        rustsecp256k1_v0_4_1_callback_call(&ctx->illegal_callback, #cond); \
        return 0; \
    } \
} while(0)

#define ARG_CHECK_NO_RETURN(cond) do { \
    if (EXPECT(!(cond), 0)) { \
        rustsecp256k1_v0_4_1_callback_call(&ctx->illegal_callback, #cond); \
    } \
} while(0)

#ifndef USE_EXTERNAL_DEFAULT_CALLBACKS
#include <stdlib.h>
#include <stdio.h>
static void rustsecp256k1_v0_4_1_default_illegal_callback_fn(const char* str, void* data) {
    (void)data;
    fprintf(stderr, "[libsecp256k1] illegal argument: %s\n", str);
    abort();
}
static void rustsecp256k1_v0_4_1_default_error_callback_fn(const char* str, void* data) {
    (void)data;
    fprintf(stderr, "[libsecp256k1] internal consistency check failed: %s\n", str);
    abort();
}
#else
void rustsecp256k1_v0_4_1_default_illegal_callback_fn(const char* str, void* data);
void rustsecp256k1_v0_4_1_default_error_callback_fn(const char* str, void* data);
#endif

static const rustsecp256k1_v0_4_1_callback default_illegal_callback = {
    rustsecp256k1_v0_4_1_default_illegal_callback_fn,
    NULL
};

static const rustsecp256k1_v0_4_1_callback default_error_callback = {
    rustsecp256k1_v0_4_1_default_error_callback_fn,
    NULL
};

struct rustsecp256k1_v0_4_1_context_struct {
    rustsecp256k1_v0_4_1_ecmult_context ecmult_ctx;
    rustsecp256k1_v0_4_1_ecmult_gen_context ecmult_gen_ctx;
    rustsecp256k1_v0_4_1_callback illegal_callback;
    rustsecp256k1_v0_4_1_callback error_callback;
    int declassify;
};

static const rustsecp256k1_v0_4_1_context rustsecp256k1_v0_4_1_context_no_precomp_ = {
    { 0 },
    { 0 },
    { rustsecp256k1_v0_4_1_default_illegal_callback_fn, 0 },
    { rustsecp256k1_v0_4_1_default_error_callback_fn, 0 },
    0
};
const rustsecp256k1_v0_4_1_context *rustsecp256k1_v0_4_1_context_no_precomp = &rustsecp256k1_v0_4_1_context_no_precomp_;

size_t rustsecp256k1_v0_4_1_context_preallocated_size(unsigned int flags) {
    size_t ret = ROUND_TO_ALIGN(sizeof(rustsecp256k1_v0_4_1_context));
    /* A return value of 0 is reserved as an indicator for errors when we call this function internally. */
    VERIFY_CHECK(ret != 0);

    if (EXPECT((flags & SECP256K1_FLAGS_TYPE_MASK) != SECP256K1_FLAGS_TYPE_CONTEXT, 0)) {
            rustsecp256k1_v0_4_1_callback_call(&default_illegal_callback,
                                    "Invalid flags");
            return 0;
    }

    if (flags & SECP256K1_FLAGS_BIT_CONTEXT_SIGN) {
        ret += SECP256K1_ECMULT_GEN_CONTEXT_PREALLOCATED_SIZE;
    }
    if (flags & SECP256K1_FLAGS_BIT_CONTEXT_VERIFY) {
        ret += SECP256K1_ECMULT_CONTEXT_PREALLOCATED_SIZE;
    }
    return ret;
}

size_t rustsecp256k1_v0_4_1_context_preallocated_clone_size(const rustsecp256k1_v0_4_1_context* ctx) {
    size_t ret = ROUND_TO_ALIGN(sizeof(rustsecp256k1_v0_4_1_context));
    VERIFY_CHECK(ctx != NULL);
    if (rustsecp256k1_v0_4_1_ecmult_gen_context_is_built(&ctx->ecmult_gen_ctx)) {
        ret += SECP256K1_ECMULT_GEN_CONTEXT_PREALLOCATED_SIZE;
    }
    if (rustsecp256k1_v0_4_1_ecmult_context_is_built(&ctx->ecmult_ctx)) {
        ret += SECP256K1_ECMULT_CONTEXT_PREALLOCATED_SIZE;
    }
    return ret;
}

rustsecp256k1_v0_4_1_context* rustsecp256k1_v0_4_1_context_preallocated_create(void* prealloc, unsigned int flags) {
    void* const base = prealloc;
    size_t prealloc_size;
    rustsecp256k1_v0_4_1_context* ret;

    if (!rustsecp256k1_v0_4_1_selftest()) {
        rustsecp256k1_v0_4_1_callback_call(&default_error_callback, "self test failed");
    }

    prealloc_size = rustsecp256k1_v0_4_1_context_preallocated_size(flags);
    if (prealloc_size == 0) {
        return NULL;
    }
    VERIFY_CHECK(prealloc != NULL);
    ret = (rustsecp256k1_v0_4_1_context*)manual_alloc(&prealloc, sizeof(rustsecp256k1_v0_4_1_context), base, prealloc_size);
    ret->illegal_callback = default_illegal_callback;
    ret->error_callback = default_error_callback;

    rustsecp256k1_v0_4_1_ecmult_context_init(&ret->ecmult_ctx);
    rustsecp256k1_v0_4_1_ecmult_gen_context_init(&ret->ecmult_gen_ctx);

    /* Flags have been checked by rustsecp256k1_v0_4_1_context_preallocated_size. */
    VERIFY_CHECK((flags & SECP256K1_FLAGS_TYPE_MASK) == SECP256K1_FLAGS_TYPE_CONTEXT);
    if (flags & SECP256K1_FLAGS_BIT_CONTEXT_SIGN) {
        rustsecp256k1_v0_4_1_ecmult_gen_context_build(&ret->ecmult_gen_ctx, &prealloc);
    }
    if (flags & SECP256K1_FLAGS_BIT_CONTEXT_VERIFY) {
        rustsecp256k1_v0_4_1_ecmult_context_build(&ret->ecmult_ctx, &prealloc);
    }
    ret->declassify = !!(flags & SECP256K1_FLAGS_BIT_CONTEXT_DECLASSIFY);

    return (rustsecp256k1_v0_4_1_context*) ret;
}

rustsecp256k1_v0_4_1_context* rustsecp256k1_v0_4_1_context_preallocated_clone(const rustsecp256k1_v0_4_1_context* ctx, void* prealloc) {
    size_t prealloc_size;
    rustsecp256k1_v0_4_1_context* ret;
    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(prealloc != NULL);

    prealloc_size = rustsecp256k1_v0_4_1_context_preallocated_clone_size(ctx);
    ret = (rustsecp256k1_v0_4_1_context*)prealloc;
    memcpy(ret, ctx, prealloc_size);
    rustsecp256k1_v0_4_1_ecmult_gen_context_finalize_memcpy(&ret->ecmult_gen_ctx, &ctx->ecmult_gen_ctx);
    rustsecp256k1_v0_4_1_ecmult_context_finalize_memcpy(&ret->ecmult_ctx, &ctx->ecmult_ctx);
    return ret;
}

void rustsecp256k1_v0_4_1_context_preallocated_destroy(rustsecp256k1_v0_4_1_context* ctx) {
    ARG_CHECK_NO_RETURN(ctx != rustsecp256k1_v0_4_1_context_no_precomp);
    if (ctx != NULL) {
        rustsecp256k1_v0_4_1_ecmult_context_clear(&ctx->ecmult_ctx);
        rustsecp256k1_v0_4_1_ecmult_gen_context_clear(&ctx->ecmult_gen_ctx);
    }
}

void rustsecp256k1_v0_4_1_context_set_illegal_callback(rustsecp256k1_v0_4_1_context* ctx, void (*fun)(const char* message, void* data), const void* data) {
    ARG_CHECK_NO_RETURN(ctx != rustsecp256k1_v0_4_1_context_no_precomp);
    if (fun == NULL) {
        fun = rustsecp256k1_v0_4_1_default_illegal_callback_fn;
    }
    ctx->illegal_callback.fn = fun;
    ctx->illegal_callback.data = data;
}

void rustsecp256k1_v0_4_1_context_set_error_callback(rustsecp256k1_v0_4_1_context* ctx, void (*fun)(const char* message, void* data), const void* data) {
    ARG_CHECK_NO_RETURN(ctx != rustsecp256k1_v0_4_1_context_no_precomp);
    if (fun == NULL) {
        fun = rustsecp256k1_v0_4_1_default_error_callback_fn;
    }
    ctx->error_callback.fn = fun;
    ctx->error_callback.data = data;
}

/* Mark memory as no-longer-secret for the purpose of analysing constant-time behaviour
 *  of the software. This is setup for use with valgrind but could be substituted with
 *  the appropriate instrumentation for other analysis tools.
 */
static SECP256K1_INLINE void rustsecp256k1_v0_4_1_declassify(const rustsecp256k1_v0_4_1_context* ctx, const void *p, size_t len) {
#if defined(VALGRIND)
    if (EXPECT(ctx->declassify,0)) VALGRIND_MAKE_MEM_DEFINED(p, len);
#else
    (void)ctx;
    (void)p;
    (void)len;
#endif
}

static int rustsecp256k1_v0_4_1_pubkey_load(const rustsecp256k1_v0_4_1_context* ctx, rustsecp256k1_v0_4_1_ge* ge, const rustsecp256k1_v0_4_1_pubkey* pubkey) {
    if (sizeof(rustsecp256k1_v0_4_1_ge_storage) == 64) {
        /* When the rustsecp256k1_v0_4_1_ge_storage type is exactly 64 byte, use its
         * representation inside rustsecp256k1_v0_4_1_pubkey, as conversion is very fast.
         * Note that rustsecp256k1_v0_4_1_pubkey_save must use the same representation. */
        rustsecp256k1_v0_4_1_ge_storage s;
        memcpy(&s, &pubkey->data[0], sizeof(s));
        rustsecp256k1_v0_4_1_ge_from_storage(ge, &s);
    } else {
        /* Otherwise, fall back to 32-byte big endian for X and Y. */
        rustsecp256k1_v0_4_1_fe x, y;
        rustsecp256k1_v0_4_1_fe_set_b32(&x, pubkey->data);
        rustsecp256k1_v0_4_1_fe_set_b32(&y, pubkey->data + 32);
        rustsecp256k1_v0_4_1_ge_set_xy(ge, &x, &y);
    }
    ARG_CHECK(!rustsecp256k1_v0_4_1_fe_is_zero(&ge->x));
    return 1;
}

static void rustsecp256k1_v0_4_1_pubkey_save(rustsecp256k1_v0_4_1_pubkey* pubkey, rustsecp256k1_v0_4_1_ge* ge) {
    if (sizeof(rustsecp256k1_v0_4_1_ge_storage) == 64) {
        rustsecp256k1_v0_4_1_ge_storage s;
        rustsecp256k1_v0_4_1_ge_to_storage(&s, ge);
        memcpy(&pubkey->data[0], &s, sizeof(s));
    } else {
        VERIFY_CHECK(!rustsecp256k1_v0_4_1_ge_is_infinity(ge));
        rustsecp256k1_v0_4_1_fe_normalize_var(&ge->x);
        rustsecp256k1_v0_4_1_fe_normalize_var(&ge->y);
        rustsecp256k1_v0_4_1_fe_get_b32(pubkey->data, &ge->x);
        rustsecp256k1_v0_4_1_fe_get_b32(pubkey->data + 32, &ge->y);
    }
}

int rustsecp256k1_v0_4_1_ec_pubkey_parse(const rustsecp256k1_v0_4_1_context* ctx, rustsecp256k1_v0_4_1_pubkey* pubkey, const unsigned char *input, size_t inputlen) {
    rustsecp256k1_v0_4_1_ge Q;

    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(pubkey != NULL);
    memset(pubkey, 0, sizeof(*pubkey));
    ARG_CHECK(input != NULL);
    if (!rustsecp256k1_v0_4_1_eckey_pubkey_parse(&Q, input, inputlen)) {
        return 0;
    }
    if (!rustsecp256k1_v0_4_1_ge_is_in_correct_subgroup(&Q)) {
        return 0;
    }
    rustsecp256k1_v0_4_1_pubkey_save(pubkey, &Q);
    rustsecp256k1_v0_4_1_ge_clear(&Q);
    return 1;
}

int rustsecp256k1_v0_4_1_ec_pubkey_serialize(const rustsecp256k1_v0_4_1_context* ctx, unsigned char *output, size_t *outputlen, const rustsecp256k1_v0_4_1_pubkey* pubkey, unsigned int flags) {
    rustsecp256k1_v0_4_1_ge Q;
    size_t len;
    int ret = 0;

    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(outputlen != NULL);
    ARG_CHECK(*outputlen >= ((flags & SECP256K1_FLAGS_BIT_COMPRESSION) ? 33u : 65u));
    len = *outputlen;
    *outputlen = 0;
    ARG_CHECK(output != NULL);
    memset(output, 0, len);
    ARG_CHECK(pubkey != NULL);
    ARG_CHECK((flags & SECP256K1_FLAGS_TYPE_MASK) == SECP256K1_FLAGS_TYPE_COMPRESSION);
    if (rustsecp256k1_v0_4_1_pubkey_load(ctx, &Q, pubkey)) {
        ret = rustsecp256k1_v0_4_1_eckey_pubkey_serialize(&Q, output, &len, flags & SECP256K1_FLAGS_BIT_COMPRESSION);
        if (ret) {
            *outputlen = len;
        }
    }
    return ret;
}

int rustsecp256k1_v0_4_1_ec_pubkey_cmp(const rustsecp256k1_v0_4_1_context* ctx, const rustsecp256k1_v0_4_1_pubkey* pubkey0, const rustsecp256k1_v0_4_1_pubkey* pubkey1) {
    unsigned char out[2][33];
    const rustsecp256k1_v0_4_1_pubkey* pk[2];
    int i;

    VERIFY_CHECK(ctx != NULL);
    pk[0] = pubkey0; pk[1] = pubkey1;
    for (i = 0; i < 2; i++) {
        size_t out_size = sizeof(out[i]);
        /* If the public key is NULL or invalid, ec_pubkey_serialize will call
         * the illegal_callback and return 0. In that case we will serialize the
         * key as all zeros which is less than any valid public key. This
         * results in consistent comparisons even if NULL or invalid pubkeys are
         * involved and prevents edge cases such as sorting algorithms that use
         * this function and do not terminate as a result. */
        if (!rustsecp256k1_v0_4_1_ec_pubkey_serialize(ctx, out[i], &out_size, pk[i], SECP256K1_EC_COMPRESSED)) {
            /* Note that ec_pubkey_serialize should already set the output to
             * zero in that case, but it's not guaranteed by the API, we can't
             * test it and writing a VERIFY_CHECK is more complex than
             * explicitly memsetting (again). */
            memset(out[i], 0, sizeof(out[i]));
        }
    }
    return rustsecp256k1_v0_4_1_memcmp_var(out[0], out[1], sizeof(out[0]));
}

static void rustsecp256k1_v0_4_1_ecdsa_signature_load(const rustsecp256k1_v0_4_1_context* ctx, rustsecp256k1_v0_4_1_scalar* r, rustsecp256k1_v0_4_1_scalar* s, const rustsecp256k1_v0_4_1_ecdsa_signature* sig) {
    (void)ctx;
    if (sizeof(rustsecp256k1_v0_4_1_scalar) == 32) {
        /* When the rustsecp256k1_v0_4_1_scalar type is exactly 32 byte, use its
         * representation inside rustsecp256k1_v0_4_1_ecdsa_signature, as conversion is very fast.
         * Note that rustsecp256k1_v0_4_1_ecdsa_signature_save must use the same representation. */
        memcpy(r, &sig->data[0], 32);
        memcpy(s, &sig->data[32], 32);
    } else {
        rustsecp256k1_v0_4_1_scalar_set_b32(r, &sig->data[0], NULL);
        rustsecp256k1_v0_4_1_scalar_set_b32(s, &sig->data[32], NULL);
    }
}

static void rustsecp256k1_v0_4_1_ecdsa_signature_save(rustsecp256k1_v0_4_1_ecdsa_signature* sig, const rustsecp256k1_v0_4_1_scalar* r, const rustsecp256k1_v0_4_1_scalar* s) {
    if (sizeof(rustsecp256k1_v0_4_1_scalar) == 32) {
        memcpy(&sig->data[0], r, 32);
        memcpy(&sig->data[32], s, 32);
    } else {
        rustsecp256k1_v0_4_1_scalar_get_b32(&sig->data[0], r);
        rustsecp256k1_v0_4_1_scalar_get_b32(&sig->data[32], s);
    }
}

int rustsecp256k1_v0_4_1_ecdsa_signature_parse_der(const rustsecp256k1_v0_4_1_context* ctx, rustsecp256k1_v0_4_1_ecdsa_signature* sig, const unsigned char *input, size_t inputlen) {
    rustsecp256k1_v0_4_1_scalar r, s;

    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(sig != NULL);
    ARG_CHECK(input != NULL);

    if (rustsecp256k1_v0_4_1_ecdsa_sig_parse(&r, &s, input, inputlen)) {
        rustsecp256k1_v0_4_1_ecdsa_signature_save(sig, &r, &s);
        return 1;
    } else {
        memset(sig, 0, sizeof(*sig));
        return 0;
    }
}

int rustsecp256k1_v0_4_1_ecdsa_signature_parse_compact(const rustsecp256k1_v0_4_1_context* ctx, rustsecp256k1_v0_4_1_ecdsa_signature* sig, const unsigned char *input64) {
    rustsecp256k1_v0_4_1_scalar r, s;
    int ret = 1;
    int overflow = 0;

    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(sig != NULL);
    ARG_CHECK(input64 != NULL);

    rustsecp256k1_v0_4_1_scalar_set_b32(&r, &input64[0], &overflow);
    ret &= !overflow;
    rustsecp256k1_v0_4_1_scalar_set_b32(&s, &input64[32], &overflow);
    ret &= !overflow;
    if (ret) {
        rustsecp256k1_v0_4_1_ecdsa_signature_save(sig, &r, &s);
    } else {
        memset(sig, 0, sizeof(*sig));
    }
    return ret;
}

int rustsecp256k1_v0_4_1_ecdsa_signature_serialize_der(const rustsecp256k1_v0_4_1_context* ctx, unsigned char *output, size_t *outputlen, const rustsecp256k1_v0_4_1_ecdsa_signature* sig) {
    rustsecp256k1_v0_4_1_scalar r, s;

    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(output != NULL);
    ARG_CHECK(outputlen != NULL);
    ARG_CHECK(sig != NULL);

    rustsecp256k1_v0_4_1_ecdsa_signature_load(ctx, &r, &s, sig);
    return rustsecp256k1_v0_4_1_ecdsa_sig_serialize(output, outputlen, &r, &s);
}

int rustsecp256k1_v0_4_1_ecdsa_signature_serialize_compact(const rustsecp256k1_v0_4_1_context* ctx, unsigned char *output64, const rustsecp256k1_v0_4_1_ecdsa_signature* sig) {
    rustsecp256k1_v0_4_1_scalar r, s;

    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(output64 != NULL);
    ARG_CHECK(sig != NULL);

    rustsecp256k1_v0_4_1_ecdsa_signature_load(ctx, &r, &s, sig);
    rustsecp256k1_v0_4_1_scalar_get_b32(&output64[0], &r);
    rustsecp256k1_v0_4_1_scalar_get_b32(&output64[32], &s);
    return 1;
}

int rustsecp256k1_v0_4_1_ecdsa_signature_normalize(const rustsecp256k1_v0_4_1_context* ctx, rustsecp256k1_v0_4_1_ecdsa_signature *sigout, const rustsecp256k1_v0_4_1_ecdsa_signature *sigin) {
    rustsecp256k1_v0_4_1_scalar r, s;
    int ret = 0;

    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(sigin != NULL);

    rustsecp256k1_v0_4_1_ecdsa_signature_load(ctx, &r, &s, sigin);
    ret = rustsecp256k1_v0_4_1_scalar_is_high(&s);
    if (sigout != NULL) {
        if (ret) {
            rustsecp256k1_v0_4_1_scalar_negate(&s, &s);
        }
        rustsecp256k1_v0_4_1_ecdsa_signature_save(sigout, &r, &s);
    }

    return ret;
}

int rustsecp256k1_v0_4_1_ecdsa_verify(const rustsecp256k1_v0_4_1_context* ctx, const rustsecp256k1_v0_4_1_ecdsa_signature *sig, const unsigned char *msghash32, const rustsecp256k1_v0_4_1_pubkey *pubkey) {
    rustsecp256k1_v0_4_1_ge q;
    rustsecp256k1_v0_4_1_scalar r, s;
    rustsecp256k1_v0_4_1_scalar m;
    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(rustsecp256k1_v0_4_1_ecmult_context_is_built(&ctx->ecmult_ctx));
    ARG_CHECK(msghash32 != NULL);
    ARG_CHECK(sig != NULL);
    ARG_CHECK(pubkey != NULL);

    rustsecp256k1_v0_4_1_scalar_set_b32(&m, msghash32, NULL);
    rustsecp256k1_v0_4_1_ecdsa_signature_load(ctx, &r, &s, sig);
    return (!rustsecp256k1_v0_4_1_scalar_is_high(&s) &&
            rustsecp256k1_v0_4_1_pubkey_load(ctx, &q, pubkey) &&
            rustsecp256k1_v0_4_1_ecdsa_sig_verify(&ctx->ecmult_ctx, &r, &s, &q, &m));
}

static SECP256K1_INLINE void buffer_append(unsigned char *buf, unsigned int *offset, const void *data, unsigned int len) {
    memcpy(buf + *offset, data, len);
    *offset += len;
}

static int nonce_function_rfc6979(unsigned char *nonce32, const unsigned char *msg32, const unsigned char *key32, const unsigned char *algo16, void *data, unsigned int counter) {
   unsigned char keydata[112];
   unsigned int offset = 0;
   rustsecp256k1_v0_4_1_rfc6979_hmac_sha256 rng;
   unsigned int i;
   /* We feed a byte array to the PRNG as input, consisting of:
    * - the private key (32 bytes) and message (32 bytes), see RFC 6979 3.2d.
    * - optionally 32 extra bytes of data, see RFC 6979 3.6 Additional Data.
    * - optionally 16 extra bytes with the algorithm name.
    * Because the arguments have distinct fixed lengths it is not possible for
    *  different argument mixtures to emulate each other and result in the same
    *  nonces.
    */
   buffer_append(keydata, &offset, key32, 32);
   buffer_append(keydata, &offset, msg32, 32);
   if (data != NULL) {
       buffer_append(keydata, &offset, data, 32);
   }
   if (algo16 != NULL) {
       buffer_append(keydata, &offset, algo16, 16);
   }
   rustsecp256k1_v0_4_1_rfc6979_hmac_sha256_initialize(&rng, keydata, offset);
   memset(keydata, 0, sizeof(keydata));
   for (i = 0; i <= counter; i++) {
       rustsecp256k1_v0_4_1_rfc6979_hmac_sha256_generate(&rng, nonce32, 32);
   }
   rustsecp256k1_v0_4_1_rfc6979_hmac_sha256_finalize(&rng);
   return 1;
}

const rustsecp256k1_v0_4_1_nonce_function rustsecp256k1_v0_4_1_nonce_function_rfc6979 = nonce_function_rfc6979;
const rustsecp256k1_v0_4_1_nonce_function rustsecp256k1_v0_4_1_nonce_function_default = nonce_function_rfc6979;

static int rustsecp256k1_v0_4_1_ecdsa_sign_inner(const rustsecp256k1_v0_4_1_context* ctx, rustsecp256k1_v0_4_1_scalar* r, rustsecp256k1_v0_4_1_scalar* s, int* recid, const unsigned char *msg32, const unsigned char *seckey, rustsecp256k1_v0_4_1_nonce_function noncefp, const void* noncedata) {
    rustsecp256k1_v0_4_1_scalar sec, non, msg;
    int ret = 0;
    int is_sec_valid;
    unsigned char nonce32[32];
    unsigned int count = 0;
    /* Default initialization here is important so we won't pass uninit values to the cmov in the end */
    *r = rustsecp256k1_v0_4_1_scalar_zero;
    *s = rustsecp256k1_v0_4_1_scalar_zero;
    if (recid) {
        *recid = 0;
    }
    if (noncefp == NULL) {
        noncefp = rustsecp256k1_v0_4_1_nonce_function_default;
    }

    /* Fail if the secret key is invalid. */
    is_sec_valid = rustsecp256k1_v0_4_1_scalar_set_b32_seckey(&sec, seckey);
    rustsecp256k1_v0_4_1_scalar_cmov(&sec, &rustsecp256k1_v0_4_1_scalar_one, !is_sec_valid);
    rustsecp256k1_v0_4_1_scalar_set_b32(&msg, msg32, NULL);
    while (1) {
        int is_nonce_valid;
        ret = !!noncefp(nonce32, msg32, seckey, NULL, (void*)noncedata, count);
        if (!ret) {
            break;
        }
        is_nonce_valid = rustsecp256k1_v0_4_1_scalar_set_b32_seckey(&non, nonce32);
        /* The nonce is still secret here, but it being invalid is is less likely than 1:2^255. */
        rustsecp256k1_v0_4_1_declassify(ctx, &is_nonce_valid, sizeof(is_nonce_valid));
        if (is_nonce_valid) {
            ret = rustsecp256k1_v0_4_1_ecdsa_sig_sign(&ctx->ecmult_gen_ctx, r, s, &sec, &msg, &non, recid);
            /* The final signature is no longer a secret, nor is the fact that we were successful or not. */
            rustsecp256k1_v0_4_1_declassify(ctx, &ret, sizeof(ret));
            if (ret) {
                break;
            }
        }
        count++;
    }
    /* We don't want to declassify is_sec_valid and therefore the range of
     * seckey. As a result is_sec_valid is included in ret only after ret was
     * used as a branching variable. */
    ret &= is_sec_valid;
    memset(nonce32, 0, 32);
    rustsecp256k1_v0_4_1_scalar_clear(&msg);
    rustsecp256k1_v0_4_1_scalar_clear(&non);
    rustsecp256k1_v0_4_1_scalar_clear(&sec);
    rustsecp256k1_v0_4_1_scalar_cmov(r, &rustsecp256k1_v0_4_1_scalar_zero, !ret);
    rustsecp256k1_v0_4_1_scalar_cmov(s, &rustsecp256k1_v0_4_1_scalar_zero, !ret);
    if (recid) {
        const int zero = 0;
        rustsecp256k1_v0_4_1_int_cmov(recid, &zero, !ret);
    }
    return ret;
}

int rustsecp256k1_v0_4_1_ecdsa_sign(const rustsecp256k1_v0_4_1_context* ctx, rustsecp256k1_v0_4_1_ecdsa_signature *signature, const unsigned char *msghash32, const unsigned char *seckey, rustsecp256k1_v0_4_1_nonce_function noncefp, const void* noncedata) {
    rustsecp256k1_v0_4_1_scalar r, s;
    int ret;
    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(rustsecp256k1_v0_4_1_ecmult_gen_context_is_built(&ctx->ecmult_gen_ctx));
    ARG_CHECK(msghash32 != NULL);
    ARG_CHECK(signature != NULL);
    ARG_CHECK(seckey != NULL);

    ret = rustsecp256k1_v0_4_1_ecdsa_sign_inner(ctx, &r, &s, NULL, msghash32, seckey, noncefp, noncedata);
    rustsecp256k1_v0_4_1_ecdsa_signature_save(signature, &r, &s);
    return ret;
}

int rustsecp256k1_v0_4_1_ec_seckey_verify(const rustsecp256k1_v0_4_1_context* ctx, const unsigned char *seckey) {
    rustsecp256k1_v0_4_1_scalar sec;
    int ret;
    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(seckey != NULL);

    ret = rustsecp256k1_v0_4_1_scalar_set_b32_seckey(&sec, seckey);
    rustsecp256k1_v0_4_1_scalar_clear(&sec);
    return ret;
}

static int rustsecp256k1_v0_4_1_ec_pubkey_create_helper(const rustsecp256k1_v0_4_1_ecmult_gen_context *ecmult_gen_ctx, rustsecp256k1_v0_4_1_scalar *seckey_scalar, rustsecp256k1_v0_4_1_ge *p, const unsigned char *seckey) {
    rustsecp256k1_v0_4_1_gej pj;
    int ret;

    ret = rustsecp256k1_v0_4_1_scalar_set_b32_seckey(seckey_scalar, seckey);
    rustsecp256k1_v0_4_1_scalar_cmov(seckey_scalar, &rustsecp256k1_v0_4_1_scalar_one, !ret);

    rustsecp256k1_v0_4_1_ecmult_gen(ecmult_gen_ctx, &pj, seckey_scalar);
    rustsecp256k1_v0_4_1_ge_set_gej(p, &pj);
    return ret;
}

int rustsecp256k1_v0_4_1_ec_pubkey_create(const rustsecp256k1_v0_4_1_context* ctx, rustsecp256k1_v0_4_1_pubkey *pubkey, const unsigned char *seckey) {
    rustsecp256k1_v0_4_1_ge p;
    rustsecp256k1_v0_4_1_scalar seckey_scalar;
    int ret = 0;
    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(pubkey != NULL);
    memset(pubkey, 0, sizeof(*pubkey));
    ARG_CHECK(rustsecp256k1_v0_4_1_ecmult_gen_context_is_built(&ctx->ecmult_gen_ctx));
    ARG_CHECK(seckey != NULL);

    ret = rustsecp256k1_v0_4_1_ec_pubkey_create_helper(&ctx->ecmult_gen_ctx, &seckey_scalar, &p, seckey);
    rustsecp256k1_v0_4_1_pubkey_save(pubkey, &p);
    rustsecp256k1_v0_4_1_memczero(pubkey, sizeof(*pubkey), !ret);

    rustsecp256k1_v0_4_1_scalar_clear(&seckey_scalar);
    return ret;
}

int rustsecp256k1_v0_4_1_ec_seckey_negate(const rustsecp256k1_v0_4_1_context* ctx, unsigned char *seckey) {
    rustsecp256k1_v0_4_1_scalar sec;
    int ret = 0;
    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(seckey != NULL);

    ret = rustsecp256k1_v0_4_1_scalar_set_b32_seckey(&sec, seckey);
    rustsecp256k1_v0_4_1_scalar_cmov(&sec, &rustsecp256k1_v0_4_1_scalar_zero, !ret);
    rustsecp256k1_v0_4_1_scalar_negate(&sec, &sec);
    rustsecp256k1_v0_4_1_scalar_get_b32(seckey, &sec);

    rustsecp256k1_v0_4_1_scalar_clear(&sec);
    return ret;
}

int rustsecp256k1_v0_4_1_ec_privkey_negate(const rustsecp256k1_v0_4_1_context* ctx, unsigned char *seckey) {
    return rustsecp256k1_v0_4_1_ec_seckey_negate(ctx, seckey);
}

int rustsecp256k1_v0_4_1_ec_pubkey_negate(const rustsecp256k1_v0_4_1_context* ctx, rustsecp256k1_v0_4_1_pubkey *pubkey) {
    int ret = 0;
    rustsecp256k1_v0_4_1_ge p;
    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(pubkey != NULL);

    ret = rustsecp256k1_v0_4_1_pubkey_load(ctx, &p, pubkey);
    memset(pubkey, 0, sizeof(*pubkey));
    if (ret) {
        rustsecp256k1_v0_4_1_ge_neg(&p, &p);
        rustsecp256k1_v0_4_1_pubkey_save(pubkey, &p);
    }
    return ret;
}


static int rustsecp256k1_v0_4_1_ec_seckey_tweak_add_helper(rustsecp256k1_v0_4_1_scalar *sec, const unsigned char *tweak32) {
    rustsecp256k1_v0_4_1_scalar term;
    int overflow = 0;
    int ret = 0;

    rustsecp256k1_v0_4_1_scalar_set_b32(&term, tweak32, &overflow);
    ret = (!overflow) & rustsecp256k1_v0_4_1_eckey_privkey_tweak_add(sec, &term);
    rustsecp256k1_v0_4_1_scalar_clear(&term);
    return ret;
}

int rustsecp256k1_v0_4_1_ec_seckey_tweak_add(const rustsecp256k1_v0_4_1_context* ctx, unsigned char *seckey, const unsigned char *tweak32) {
    rustsecp256k1_v0_4_1_scalar sec;
    int ret = 0;
    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(seckey != NULL);
    ARG_CHECK(tweak32 != NULL);

    ret = rustsecp256k1_v0_4_1_scalar_set_b32_seckey(&sec, seckey);
    ret &= rustsecp256k1_v0_4_1_ec_seckey_tweak_add_helper(&sec, tweak32);
    rustsecp256k1_v0_4_1_scalar_cmov(&sec, &rustsecp256k1_v0_4_1_scalar_zero, !ret);
    rustsecp256k1_v0_4_1_scalar_get_b32(seckey, &sec);

    rustsecp256k1_v0_4_1_scalar_clear(&sec);
    return ret;
}

int rustsecp256k1_v0_4_1_ec_privkey_tweak_add(const rustsecp256k1_v0_4_1_context* ctx, unsigned char *seckey, const unsigned char *tweak32) {
    return rustsecp256k1_v0_4_1_ec_seckey_tweak_add(ctx, seckey, tweak32);
}

static int rustsecp256k1_v0_4_1_ec_pubkey_tweak_add_helper(const rustsecp256k1_v0_4_1_ecmult_context* ecmult_ctx, rustsecp256k1_v0_4_1_ge *p, const unsigned char *tweak32) {
    rustsecp256k1_v0_4_1_scalar term;
    int overflow = 0;
    rustsecp256k1_v0_4_1_scalar_set_b32(&term, tweak32, &overflow);
    return !overflow && rustsecp256k1_v0_4_1_eckey_pubkey_tweak_add(ecmult_ctx, p, &term);
}

int rustsecp256k1_v0_4_1_ec_pubkey_tweak_add(const rustsecp256k1_v0_4_1_context* ctx, rustsecp256k1_v0_4_1_pubkey *pubkey, const unsigned char *tweak32) {
    rustsecp256k1_v0_4_1_ge p;
    int ret = 0;
    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(rustsecp256k1_v0_4_1_ecmult_context_is_built(&ctx->ecmult_ctx));
    ARG_CHECK(pubkey != NULL);
    ARG_CHECK(tweak32 != NULL);

    ret = rustsecp256k1_v0_4_1_pubkey_load(ctx, &p, pubkey);
    memset(pubkey, 0, sizeof(*pubkey));
    ret = ret && rustsecp256k1_v0_4_1_ec_pubkey_tweak_add_helper(&ctx->ecmult_ctx, &p, tweak32);
    if (ret) {
        rustsecp256k1_v0_4_1_pubkey_save(pubkey, &p);
    }

    return ret;
}

int rustsecp256k1_v0_4_1_ec_seckey_tweak_mul(const rustsecp256k1_v0_4_1_context* ctx, unsigned char *seckey, const unsigned char *tweak32) {
    rustsecp256k1_v0_4_1_scalar factor;
    rustsecp256k1_v0_4_1_scalar sec;
    int ret = 0;
    int overflow = 0;
    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(seckey != NULL);
    ARG_CHECK(tweak32 != NULL);

    rustsecp256k1_v0_4_1_scalar_set_b32(&factor, tweak32, &overflow);
    ret = rustsecp256k1_v0_4_1_scalar_set_b32_seckey(&sec, seckey);
    ret &= (!overflow) & rustsecp256k1_v0_4_1_eckey_privkey_tweak_mul(&sec, &factor);
    rustsecp256k1_v0_4_1_scalar_cmov(&sec, &rustsecp256k1_v0_4_1_scalar_zero, !ret);
    rustsecp256k1_v0_4_1_scalar_get_b32(seckey, &sec);

    rustsecp256k1_v0_4_1_scalar_clear(&sec);
    rustsecp256k1_v0_4_1_scalar_clear(&factor);
    return ret;
}

int rustsecp256k1_v0_4_1_ec_privkey_tweak_mul(const rustsecp256k1_v0_4_1_context* ctx, unsigned char *seckey, const unsigned char *tweak32) {
    return rustsecp256k1_v0_4_1_ec_seckey_tweak_mul(ctx, seckey, tweak32);
}

int rustsecp256k1_v0_4_1_ec_pubkey_tweak_mul(const rustsecp256k1_v0_4_1_context* ctx, rustsecp256k1_v0_4_1_pubkey *pubkey, const unsigned char *tweak32) {
    rustsecp256k1_v0_4_1_ge p;
    rustsecp256k1_v0_4_1_scalar factor;
    int ret = 0;
    int overflow = 0;
    VERIFY_CHECK(ctx != NULL);
    ARG_CHECK(rustsecp256k1_v0_4_1_ecmult_context_is_built(&ctx->ecmult_ctx));
    ARG_CHECK(pubkey != NULL);
    ARG_CHECK(tweak32 != NULL);

    rustsecp256k1_v0_4_1_scalar_set_b32(&factor, tweak32, &overflow);
    ret = !overflow && rustsecp256k1_v0_4_1_pubkey_load(ctx, &p, pubkey);
    memset(pubkey, 0, sizeof(*pubkey));
    if (ret) {
        if (rustsecp256k1_v0_4_1_eckey_pubkey_tweak_mul(&ctx->ecmult_ctx, &p, &factor)) {
            rustsecp256k1_v0_4_1_pubkey_save(pubkey, &p);
        } else {
            ret = 0;
        }
    }

    return ret;
}

int rustsecp256k1_v0_4_1_context_randomize(rustsecp256k1_v0_4_1_context* ctx, const unsigned char *seed32) {
    VERIFY_CHECK(ctx != NULL);
    if (rustsecp256k1_v0_4_1_ecmult_gen_context_is_built(&ctx->ecmult_gen_ctx)) {
        rustsecp256k1_v0_4_1_ecmult_gen_blind(&ctx->ecmult_gen_ctx, seed32);
    }
    return 1;
}

int rustsecp256k1_v0_4_1_ec_pubkey_combine(const rustsecp256k1_v0_4_1_context* ctx, rustsecp256k1_v0_4_1_pubkey *pubnonce, const rustsecp256k1_v0_4_1_pubkey * const *pubnonces, size_t n) {
    size_t i;
    rustsecp256k1_v0_4_1_gej Qj;
    rustsecp256k1_v0_4_1_ge Q;

    ARG_CHECK(pubnonce != NULL);
    memset(pubnonce, 0, sizeof(*pubnonce));
    ARG_CHECK(n >= 1);
    ARG_CHECK(pubnonces != NULL);

    rustsecp256k1_v0_4_1_gej_set_infinity(&Qj);

    for (i = 0; i < n; i++) {
        rustsecp256k1_v0_4_1_pubkey_load(ctx, &Q, pubnonces[i]);
        rustsecp256k1_v0_4_1_gej_add_ge(&Qj, &Qj, &Q);
    }
    if (rustsecp256k1_v0_4_1_gej_is_infinity(&Qj)) {
        return 0;
    }
    rustsecp256k1_v0_4_1_ge_set_gej(&Q, &Qj);
    rustsecp256k1_v0_4_1_pubkey_save(pubnonce, &Q);
    return 1;
}

#ifdef ENABLE_MODULE_ECDH
# include "modules/ecdh/main_impl.h"
#endif

#ifdef ENABLE_MODULE_RECOVERY
# include "modules/recovery/main_impl.h"
#endif

#ifdef ENABLE_MODULE_EXTRAKEYS
# include "modules/extrakeys/main_impl.h"
#endif

#ifdef ENABLE_MODULE_SCHNORRSIG
# include "modules/schnorrsig/main_impl.h"
#endif
