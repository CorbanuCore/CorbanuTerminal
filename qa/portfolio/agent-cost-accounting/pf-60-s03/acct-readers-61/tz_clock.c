// QA-only probe executes inside the exact product PID, after exec and DYLD load.
// No credential access. Probe output is limited to TZ, PID and a localtime control.
#include "../acct-boundaries-58/fixture_clock.c"
#include <unistd.h>
__attribute__((constructor)) static void record_timezone_control(void) {
    const char *path = getenv("ACCT_TZ_PROBE");
    if (!path) return;
    const char *zone = getenv("TZ");
    time_t epoch = fixture_ms() / 1000;
    struct tm local;
    char formatted[80];
    tzset();
    if (!localtime_r(&epoch, &local)) abort();
    if (!strftime(formatted, sizeof(formatted), "%Y-%m-%dT%H:%M:%S%z", &local)) abort();
    char output[4096];
    if (snprintf(output, sizeof(output), "%s.%d.json", path, getpid()) >= sizeof(output)) abort();
    FILE *file = fopen(output, "w");
    if (!file) abort();
    fprintf(file, "{\"pid\":%d,\"tz\":\"%s\",\"epoch\":%lld,\"local\":\"%s\"}\n",
            getpid(), zone ? zone : "", (long long)epoch, formatted);
    fclose(file);
}
