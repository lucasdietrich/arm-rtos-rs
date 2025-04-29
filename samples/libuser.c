/*
 * Copyright (c) 2024 Lucas Dietrich <ld.adecy@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#include <stdarg.h>

#include <libuser.h>

// syscalls
int32_t __syscall_test(uint32_t arg0, uint32_t arg1, uint32_t arg2, uint32_t arg3)
{
    register uint32_t r0 __asm__("r0") = arg0;
    register uint32_t r1 __asm__("r1") = arg1;
    register uint32_t r2 __asm__("r2") = arg2;
    register uint32_t r3 __asm__("r3") = arg3;

	__asm__ __volatile__("svc 0\n"
						 :
						 : "r"(r0), "r"(r1), "r"(r2), "r"(r3)
						 : "memory");

	return (int32_t)r0;
}

int32_t __syscall_kernel(uint32_t arg0, uint32_t arg1, uint32_t arg2, uint32_t arg3)
{
    register uint32_t r0 __asm__("r0") = arg0;
    register uint32_t r1 __asm__("r1") = arg1;
    register uint32_t r2 __asm__("r2") = arg2;
    register uint32_t r3 __asm__("r3") = arg3;

	__asm__ __volatile__("svc 1\n"
						 :
						 : "r"(r0), "r"(r1), "r"(r2), "r"(r3)
						 : "memory");

	return (int32_t)r0;
}

int32_t __syscall_io(uint32_t arg0, uint32_t arg1, uint32_t arg2, uint32_t arg3)
{
    register uint32_t r0 __asm__("r0") = arg0;
    register uint32_t r1 __asm__("r1") = arg1;
    register uint32_t r2 __asm__("r2") = arg2;
    register uint32_t r3 __asm__("r3") = arg3;

	__asm__ __volatile__("svc 2\n"
						 :
						 : "r"(r0), "r"(r1), "r"(r2), "r"(r3)
						 : "memory");

	return (int32_t)r0;
}

int32_t __syscall_driver(uint32_t arg0, uint32_t arg1, uint32_t arg2, uint32_t arg3)
{
    register uint32_t r0 __asm__("r0") = arg0;
    register uint32_t r1 __asm__("r1") = arg1;
    register uint32_t r2 __asm__("r2") = arg2;
    register uint32_t r3 __asm__("r3") = arg3;

	__asm__ __volatile__("svc 3\n"
						 :
						 : "r"(r0), "r"(r1), "r"(r2), "r"(r3)
						 : "memory");

	return (int32_t)r0;
}
