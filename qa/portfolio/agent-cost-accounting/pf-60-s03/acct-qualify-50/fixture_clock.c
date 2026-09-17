// Qualification fixture: offset wall clock only; leave monotonic timers intact.
#include <stdint.h>
#include <stdlib.h>
#include <sys/time.h>
#include <time.h>
static long offset(void) {
    const char *v = getenv("ACCT_QUALIFY_CLOCK_OFFSET_SECONDS");
    return v ? strtol(v, NULL, 10) : 0;
}
static int fixture_gettimeofday(struct timeval *tv, void *tz) {
    int result = gettimeofday(tv, tz);
    if (!result && tv) tv->tv_sec += offset();
    return result;
}
static int fixture_clock_gettime(clockid_t id, struct timespec *ts) {
    int result = clock_gettime(id, ts);
    if (!result && id == CLOCK_REALTIME) ts->tv_sec += offset();
    return result;
}
static time_t fixture_time(time_t *out) {
    time_t value = time(NULL) + offset();
    if (out) *out = value;
    return value;
}
#define INTERPOSE(replacement, original) \
__attribute__((used)) static struct { const void *new_fn; const void *old_fn; } \
interpose_##original __attribute__((section("__DATA,__interpose"))) = \
{ (const void *)(uintptr_t)&replacement, (const void *)(uintptr_t)&original };
INTERPOSE(fixture_gettimeofday, gettimeofday)
INTERPOSE(fixture_clock_gettime, clock_gettime)
INTERPOSE(fixture_time, time)
