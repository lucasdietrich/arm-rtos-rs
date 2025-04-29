/*
 * Copyright (c) 2024 Lucas Dietrich <ld.adecy@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#ifndef _user_h
#define _user_h

#include <stdint-gcc.h>

int32_t __syscall_test(uint32_t arg0, uint32_t arg1, uint32_t arg2, uint32_t arg3);
int32_t __syscall_kernel(uint32_t arg0, uint32_t arg1, uint32_t arg2, uint32_t arg3);
int32_t __syscall_io(uint32_t arg0, uint32_t arg1, uint32_t arg2, uint32_t arg3);
int32_t __syscall_driver(uint32_t arg0, uint32_t arg1, uint32_t arg2, uint32_t arg3);

#define k_yield()					   __syscall_kernel(0, 0, 0, 0)
#define k_sleep(ms)					   __syscall_kernel(ms, 0, 0, 1)
#define k_syscall_test(r0, r1, r2, r3) __syscall_test(r0, r1, r2, r3)

#endif // _libc_h
