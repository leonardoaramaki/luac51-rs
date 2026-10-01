/*
** $Id: lmem.c,v 1.70.1.1 2007/12/27 13:02:25 roberto Exp $
** Interface to Memory Manager
** Compiler-only adaptation; execution and garbage-collection support removed.
** See Copyright Notice in lua.h
*/


#include <stddef.h>
#include <stdlib.h>

#define lmem_c
#define LUA_CORE

#include "lua.h"

#include "lmem.h"
#include "lobject.h"
#include "lstate.h"



/*
** Each allocation has an aligned prefix linking it to this compilation.
** Reallocations keep that list valid even if allocation fails. Explicit
** frees unlink blocks; the compiler frees everything left on all exits.
*/



#define MINSIZEARRAY	4


void *luaM_growaux_ (lua_State *L, void *block, int *size, size_t size_elems,
                     int limit, const char *errormsg) {
  void *newblock;
  int newsize;
  if (*size >= limit/2) {  /* cannot double it? */
    if (*size >= limit)  /* cannot grow even a little? */
      luaG_runerror(L, errormsg);
    newsize = limit;  /* still have at least one free place */
  }
  else {
    newsize = (*size)*2;
    if (newsize < MINSIZEARRAY)
      newsize = MINSIZEARRAY;  /* minimum size */
  }
  newblock = luaM_reallocv(L, block, *size, newsize, size_elems);
  *size = newsize;  /* update only when everything else is OK */
  return newblock;
}


void *luaM_toobig (lua_State *L) {
  luaG_runerror(L, "memory allocation error: block too big");
  return NULL;  /* to avoid warnings */
}



/*
** generic allocation routine.
*/
void *luaM_realloc_ (lua_State *L, void *block, size_t osize, size_t nsize) {
  Allocation *old = block ? cast(Allocation *, block) - 1 : NULL;
  Allocation *prev = old ? old->links.prev : NULL;
  Allocation *next = old ? old->links.next : L->allocations;
  Allocation *allocation;
  UNUSED(osize);
  if (nsize == 0) {
    if (old != NULL) {
      if (prev) prev->links.next = next;
      else L->allocations = next;
      if (next) next->links.prev = prev;
      free(old);
    }
    return NULL;
  }
  if (nsize > MAX_SIZET - sizeof(Allocation))
    luaD_throw(L, LUA_ERRMEM);
  allocation = realloc(old, sizeof(Allocation) + nsize);
  if (allocation == NULL)
    luaD_throw(L, LUA_ERRMEM);
  allocation->links.prev = prev;
  allocation->links.next = next;
  if (prev) prev->links.next = allocation;
  else L->allocations = allocation;
  if (next) next->links.prev = allocation;
  return allocation + 1;
}
