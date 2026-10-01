#ifndef LUAC_TEST_ALLOCATIONS_H
#define LUAC_TEST_ALLOCATIONS_H

#include <stdlib.h>

void *luac_test_calloc(size_t count, size_t size);
void *luac_test_realloc(void *pointer, size_t size);
void luac_test_free(void *pointer);

#define calloc luac_test_calloc
#define realloc luac_test_realloc
#define free luac_test_free

#endif
