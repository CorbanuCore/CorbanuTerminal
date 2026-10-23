/* Synthetic x86_64 Linux fixture: no libc, secrets, configurable destination,
 * shell, or privilege change. Parent PID is rendezvous naming, NEVER auth. */
static long call(long n, long a, long b, long c) {
    long result;
    __asm__ volatile("syscall" : "=a"(result) : "a"(n), "D"(a), "S"(b), "d"(c)
                     : "rcx", "r11", "memory");
    return result;
}
static void die(long status) {
    call(60, status, 0, 0);
    __builtin_unreachable();
}
struct address { unsigned short family; char name[108]; };
struct pollfd { int fd; short events, revents; };
void start(long *stack) {
    char **argv = (char **)(stack + 1);
    if (*stack != 6 || (argv[2][0] != 'j' && argv[2][0] != 'p' && argv[2][0] != 'w')) die(90);
    struct address address = {1, {0}};
    const char prefix[] = "corbanu-pf27-";
    int len = 1;
    for (int i = 0; prefix[i]; ++i) address.name[len++] = prefix[i];
    long parent = call(110, 0, 0, 0);
    char digits[20];
    int count = 0;
    do { digits[count++] = '0' + parent % 10; parent /= 10; } while (parent);
    while (count) address.name[len++] = digits[--count];
    struct pollfd peers[2];
    for (int i = 0; i < 2; ++i) {
        long fd = call(41, 1, 1, 0);
        if (fd < 0 || call(42, fd, (long)&address, 2 + len)) die(91);
        peers[i] = (struct pollfd){(int)fd, 1, 0};
        if (call(1, fd, (long)argv[2], 1) != 1) die(92);
    }
    for (;;) {
        if (call(7, (long)peers, 2, 5000) <= 0) die(93);
        for (int i = 0; i < 2; ++i) {
            if (!peers[i].revents) continue;
            char command;
            if (call(0, peers[i].fd, (long)&command, 1) <= 0) {
                call(3, peers[i].fd, 0, 0);
                peers[i].fd = -1;
            } else if (command == 'x') {
                die(0);
            } else if (command == 'e') {
                for (int j = 0; j < 2; ++j) call(48, peers[j].fd, 1, 0);
                for (;;) call(34, 0, 0, 0);
            }
        }
    }
}
__asm__(".global _start\n_start:\nmov %rsp,%rdi\nand $-16,%rsp\ncall start\nud2\n");
