/*
 * Run from the crate directory (the output belongs in target/):
 * cc -std=c11 -g -fsanitize=address,undefined -fno-sanitize-recover=all \
 *   -include tests/native/allocations.h -Ivendor/lua-5.1.5/src \
 *   vendor/lua-5.1.5/src/l*.c tests/native/allocations.c -lm \
 *   -o target/test-allocations
 * target/test-allocations tests/fixtures/program.lua
 */
#undef calloc
#undef realloc
#undef free

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "lstate.h"

static size_t attempts;
static size_t fail_at;
static size_t live;
static size_t writer_calls;
static size_t fail_writer_at;

void *luac_test_calloc(size_t count, size_t size) {
  void *result;
  if (++attempts == fail_at) return NULL;
  result = calloc(count, size);
  if (result != NULL) live++;
  return result;
}

void *luac_test_realloc(void *pointer, size_t size) {
  int is_new = pointer == NULL;
  void *result;
  assert(size != 0);
  if (++attempts == fail_at) return NULL;
  result = realloc(pointer, size);
  if (is_new && result != NULL) live++;
  return result;
}

void luac_test_free(void *pointer) {
  if (pointer != NULL) {
    assert(live > 0);
    live--;
    free(pointer);
  }
}

static int writer(lua_State *L, const void *bytes, size_t size, void *output) {
  size_t i;
  volatile unsigned char *checksum = output;
  (void)L;
  if (++writer_calls == fail_writer_at) return 1;
  for (i = 0; i < size; i++) *checksum ^= ((const unsigned char *)bytes)[i];
  return 0;
}

static int compile(const char *source, size_t length) {
  volatile unsigned char checksum = 0;
  int status;
  attempts = 0;
  writer_calls = 0;
  status = luac_compile(source, length, "@test.lua", writer, (void *)&checksum);
  assert(live == 0);
  return status;
}

static void check(const char *source, size_t length, int expected) {
  size_t allocations, writes, i;
  fail_at = 0;
  fail_writer_at = 0;
  assert(compile(source, length) == expected);
  allocations = attempts;
  writes = writer_calls;
  for (i = 1; i <= allocations; i++) {
    fail_at = i;
    assert(compile(source, length) == LUA_ERRMEM);
  }
  fail_at = 0;
  for (i = 1; i <= writes; i++) {
    fail_writer_at = i;
    assert(compile(source, length) == LUA_ERRMEM);
  }
  fail_writer_at = 0;
  assert(compile(source, length) == expected);
  printf("OK: %zu allocation failures, %zu writer failures, no leaks\n",
         allocations, writes);
}

int main(int argc, char **argv) {
  const char *invalid[] = {
    "local function broken(a) local t = {1, 2, 3}",
    "return [=[unfinished",
    "return 1e+",
    "local = 1"
  };
  const char *numbers = "return {1e300, -1e300, 0, -0, 1/0, 0/0, 0xFF}";
  size_t i;
  check("", 0, 0);
  check(numbers, strlen(numbers), 0);
  for (i = 0; i < sizeof(invalid) / sizeof(invalid[0]); i++)
    check(invalid[i], strlen(invalid[i]), LUA_ERRSYNTAX);
  if (argc > 1) {
    FILE *file = fopen(argv[1], "rb");
    long length;
    char *source;
    assert(file != NULL);
    assert(fseek(file, 0, SEEK_END) == 0);
    length = ftell(file);
    assert(length >= 0);
    rewind(file);
    source = malloc((size_t)length + 1);
    assert(source != NULL);
    assert(fread(source, 1, (size_t)length, file) == (size_t)length);
    fclose(file);
    check(source, (size_t)length, 0);
    free(source);
  }
  return 0;
}
