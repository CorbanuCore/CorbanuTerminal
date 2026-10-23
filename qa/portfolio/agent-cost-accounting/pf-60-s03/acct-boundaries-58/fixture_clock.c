// Test-only exact wall clock. Monotonic clocks remain real.
// Each process receives an immutable timestamp from the driver.
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/time.h>
#include <time.h>
static int64_t fixture_ms(void) {
    const char *value = getenv("ACCT_QUALIFY_CLOCK_MS");
    return value ? strtoll(value, NULL, 10) : 0;
}
static int fixture_gettimeofday(struct timeval *tv, void *tz) {
    int result = gettimeofday(tv, tz);
    int64_t ms = fixture_ms();
    if (!result && tv && ms) { tv->tv_sec = ms / 1000; tv->tv_usec = ms % 1000 * 1000; }
    return result;
}
static int fixture_clock_gettime(clockid_t id, struct timespec *ts) {
    int result = clock_gettime(id, ts);
    int64_t ms = fixture_ms();
    if (!result && id == CLOCK_REALTIME && ms) {
        ts->tv_sec = ms / 1000; ts->tv_nsec = ms % 1000 * 1000000;
    }
    return result;
}
static time_t fixture_time(time_t *out) {
    int64_t ms = fixture_ms();
    time_t value = ms ? ms / 1000 : time(NULL);
    if (out) *out = value;
    return value;
}
#define INTERPOSE(replacement, original) \
__attribute__((used)) static struct { const void *new_fn; const void *old_fn; } \
interpose_##original __attribute__((section("__DATA,__interpose"))) = \
{ (const void *)(uintptr_t)&replacement, (const void *)(uintptr_t)&original };
// Keep gettimeofday real: parking_lot builds kernel absolute wait deadlines with it.
INTERPOSE(fixture_clock_gettime, clock_gettime)
INTERPOSE(fixture_time, time)
