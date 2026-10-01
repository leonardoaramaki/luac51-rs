/*
** Compiler-only entry point for the Lua 5.1.5 parser and bytecode writer.
** Prototype initialization is derived from the official lfunc.c.
** See Copyright Notice in lua.h.
*/

#include <stdarg.h>
#include <stdlib.h>
#include <string.h>

#define LUA_CORE
#include "lstate.h"
#include "llex.h"
#include "lmem.h"
#include "lparser.h"
#include "lstring.h"
#include "lundump.h"
#include "lzio.h"

void luaD_throw(lua_State *L, int status) {
  L->status = status;
  if (status == LUA_ERRMEM) L->error = MEMERRMSG;
  longjmp(L->recovery, 1);
}

void luaG_runerror(lua_State *L, const char *format, ...) {
  va_list args;
  va_start(args, format);
  luaO_pushvfstring(L, format, args);
  va_end(args);
  luaD_throw(L, LUA_ERRSYNTAX);
}

Proto *luaF_newproto(lua_State *L) {
  Proto *f = luaM_new(L, Proto);
  f->tt = LUA_TPROTO;
  f->next = NULL;
  f->k = NULL;
  f->sizek = 0;
  f->p = NULL;
  f->sizep = 0;
  f->code = NULL;
  f->sizecode = 0;
  f->sizelineinfo = 0;
  f->sizeupvalues = 0;
  f->nups = 0;
  f->upvalues = NULL;
  f->numparams = 0;
  f->is_vararg = 0;
  f->maxstacksize = 0;
  f->lineinfo = NULL;
  f->sizelocvars = 0;
  f->locvars = NULL;
  f->linedefined = 0;
  f->lastlinedefined = 0;
  f->source = NULL;
  return f;
}

typedef struct Source {
  const char *bytes;
  size_t size;
} Source;

static const char *read_source(lua_State *L, void *data, size_t *size) {
  Source *source = data;
  UNUSED(L);
  *size = source->size;
  source->size = 0;
  return source->bytes;
}

int luac_compile(const char *source, size_t size, const char *name,
                 lua_Writer writer, void *output) {
  lua_State *L = calloc(1, sizeof(*L));
  int status;
  if (L == NULL) {
    writer(NULL, MEMERRMSG, sizeof(MEMERRMSG) - 1, output);
    return LUA_ERRMEM;
  }
  /* Error recovery remains entirely in C, never across a Rust frame. */
  if (setjmp(L->recovery) == 0) {
    Source input = {source, size};
    Mbuffer buffer = {NULL, 0, 0};
    ZIO stream;
    Proto *prototype;
    luaS_resize(L, MINSTRTABSIZE);
    luaX_init(L);
    luaZ_init(L, &stream, read_source, &input);
    prototype = luaY_parser(L, &stream, &buffer, name);
    if (luaU_dump(L, prototype, writer, output, 0) != 0)
      L->status = LUA_ERRMEM;
  }
  else {
    const char *message = L->error ? L->error : "Lua compilation failed";
    if (writer(L, message, strlen(message), output) != 0)
      L->status = LUA_ERRMEM;
  }
  status = L->status;
  /* Includes every partially initialized allocation after a parser/OOM error. */
  while (L->allocations != NULL) {
    Allocation *allocation = L->allocations;
    L->allocations = allocation->links.next;
    free(allocation);
  }
  free(L);
  return status;
}
