/*
 * Copyright (c) 2024 Lucas Dietrich <ld.adecy@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#include <stdarg.h>

#include <helpers.h>
#include <libc.h>

__noinline int strlen(const char *str)
{
	int len = 0;
	while (*str++) {
		len++;
	}
	return len;
}
