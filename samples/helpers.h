/*
 * Copyright (c) 2024 Lucas Dietrich <ld.adecy@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#ifndef helpers_h
#define helpers_h

#include <stdint-gcc.h>

// strings
#define Z_STRINGIFY(x) #x
#define STRINGIFY(s)   Z_STRINGIFY(s)

#define _DO_CONCAT(x, y) x##y
#define _CONCAT(x, y)	 _DO_CONCAT(x, y)

// GCC
#define ARG_UNUSED(arg) ((void)arg)

#define Z_LINK_SECTION(_section) __attribute__((section(Z_STRINGIFY(_section))))
#define Z_LINK_SECTION_USED(_section)                                                    \
	__attribute__((used, section(Z_STRINGIFY(_section))))

#define __noinline		 __attribute__((noinline))
#define __noreturn		 __attribute__((__noreturn__))
#define CODE_UNREACHABLE __builtin_unreachable();
#define __always_inline	 __attribute__((always_inline)) inline
#define __noinit		 Z_LINK_SECTION(.noinit)
#define __bss			 Z_LINK_SECTION(.bss)
#define __packed		 __attribute__((packed))

#define __static_assert						_Static_assert
#define __STATIC_ASSERT(test_for_true, msg) __static_assert(test_for_true, msg)
#define __STATIC_ASSERT_NOMSG(test_for_true)                                             \
	__static_assert(test_for_true, "(" #test_for_true ") failed")

#define MIN(a, b)					   (((a) < (b)) ? (a) : (b))
#define MAX(a, b)					   (((a) > (b)) ? (a) : (b))
#define ARRAY_SIZE(array)			   (sizeof(array) / sizeof(array[0]))
#define CONTAINER_OF(ptr, type, field) ((type *)(((char *)(ptr)) - offsetof(type, field)))
#define SIZEOF_MEMBER(type, member)	   (sizeof(((type *)0)->member))
#define INDEX_OF(array, element)	   ((element) - (array))

#define BIT(b)		   (1llu << (b))
#define SET_BIT(x, b)  ((x) |= (b))
#define CLR_BIT(x, b)  ((x) &= (~(b)))
#define TEST_BIT(x, b) ((bool)((x) & (b)))

#endif // helpers_h
