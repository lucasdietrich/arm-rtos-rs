/*
 * Copyright (c) 2024 Lucas Dietrich <ld.adecy@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#ifndef _arm_rtos_rs_h
#define _arm_rtos_rs_h

#include <helpers.h>

#define k_yield()					   __syscall_kernel(0, 0, 0, 0)
#define k_sleep(ms)					   __syscall_kernel(ms, 0, 0, 1)
#define k_syscall_test(r0, r1, r2, r3) __syscall_test(r0, r1, r2, r3)

#endif // _arm_rtos_rs_h
