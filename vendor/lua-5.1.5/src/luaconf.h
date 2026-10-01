/*
** Compiler-only configuration derived from Lua 5.1.5 luaconf.h.
** Runtime, platform-loader and standard-library settings removed.
** See Copyright Notice in lua.h.
*/
#ifndef lconfig_h
#define lconfig_h

#include <limits.h>
#include <stddef.h>
#include <math.h>

#define LUAI_FUNC extern
#define LUAI_DATA extern
#define LUA_QL(x) "'" x "'"
#define LUA_QS LUA_QL("%s")

/* Preserve the official 5.1 language compatibility settings and limits. */
#define LUA_COMPAT_VARARG
#define LUA_COMPAT_LSTR 1
#define LUAI_MAXCCALLS 200
#define LUAI_MAXVARS 200
#define LUAI_MAXUPVALUES 60

#if INT_MAX-20 < 32760
#define LUAI_BITSINT 16
#elif INT_MAX > 2147483640L
#define LUAI_BITSINT 32
#else
#error "unsupported int size"
#endif

#if LUAI_BITSINT >= 32
#define LUAI_UINT32 unsigned int
#define LUAI_UMEM size_t
#else
#define LUAI_UINT32 unsigned long
#define LUAI_UMEM unsigned long
#endif

#define LUA_NUMBER double
#define lua_str2number(s,p) strtod((s), (p))
/* Out-of-range/NaN constants are hash keys, never integer array indices. */
#define lua_number2int(i,d) \
  ((i) = ((d) >= INT_MIN && (d) <= INT_MAX) ? (int)(d) : INT_MIN)

#define luai_numadd(a,b) ((a)+(b))
#define luai_numsub(a,b) ((a)-(b))
#define luai_nummul(a,b) ((a)*(b))
#define luai_numdiv(a,b) ((a)/(b))
#define luai_nummod(a,b) ((a) - floor((a)/(b))*(b))
#define luai_numpow(a,b) (pow(a,b))
#define luai_numunm(a) (-(a))
#define luai_numeq(a,b) ((a)==(b))
#define luai_numisnan(a) (!luai_numeq((a), (a)))

#define LUAI_USER_ALIGNMENT_T union { double u; void *s; long l; }

#endif
