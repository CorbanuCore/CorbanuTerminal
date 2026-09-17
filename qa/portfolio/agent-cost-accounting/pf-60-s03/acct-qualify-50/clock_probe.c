#include <stdio.h>
#include <sys/time.h>
#include <time.h>
int main(void) {
    struct timeval tv;
    struct timespec real, monotonic;
    gettimeofday(&tv, NULL);
    clock_gettime(CLOCK_REALTIME, &real);
    clock_gettime(CLOCK_MONOTONIC, &monotonic);
    printf("gettimeofday=%ld realtime=%ld time=%ld monotonic=%ld\n",
           tv.tv_sec, real.tv_sec, time(NULL), monotonic.tv_sec);
    return 0;
}
