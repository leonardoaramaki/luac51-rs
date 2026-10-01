/*
** Compiler-only replacement for the Lua 5.1.5 runtime state.
** See Copyright Notice in lua.h.
*/
#ifndef lstate_h
#define lstate_h

#include <setjmp.h>
#include <stddef.h>
#include "lobject.h"

typedef union Allocation Allocation;
union Allocation {
  max_align_t alignment;
  struct {
    Allocation *prev;
    Allocation *next;
  } links;
};

typedef struct stringtable {
  GCObject **hash;
  lu_int32 nuse;
  int size;
} stringtable;

/* No Lua value stack, call frames, globals, closures, hooks or coroutines. */
struct lua_State {
  stringtable strt;
  Allocation *allocations;
  jmp_buf recovery;
  const char *error;
  unsigned short nCcalls; /* parser recursion depth only */
  int status;
};

union GCObject {
  GCheader gch;
  TString ts;
  Table h;
  Proto p;
};

#define rawgco2ts(o) check_exp((o)->gch.tt == LUA_TSTRING, &((o)->ts))
#define gco2ts(o) (&rawgco2ts(o)->tsv)
#define obj2gco(v) cast(GCObject *, (v))

LUAI_FUNC void luaD_throw(lua_State *L, int status);
LUAI_FUNC void luaG_runerror(lua_State *L, const char *format, ...);
LUAI_FUNC Proto *luaF_newproto(lua_State *L);

/* Writes bytecode on success, an error message on failure; owns no output. */
int luac_compile(const char *source, size_t size, const char *name,
                 lua_Writer writer, void *output);

#endif
