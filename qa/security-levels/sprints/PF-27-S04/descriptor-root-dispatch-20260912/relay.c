/* Fixed synthetic journal/policy relays. Parent/role name rendezvous, not authority. */
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
static void send_all(int fd, const char *bytes, long count) {
    while (count) {
        long sent = call(1, fd, (long)bytes, count);
        if (sent == -4) continue;
        if (sent <= 0) die(94);
        bytes += sent;
        count -= sent;
    }
}
struct address { unsigned short family; char name[108]; };
struct pollfd { int fd; short events, revents; };
void start(long *stack) {
    char **argv = (char **)(stack + 1);
    if (*stack != 6 || (argv[2][0] != 'j' && argv[2][0] != 'p')) die(90);
    struct address address = {1, {0}};
    const char prefix[] = "corbanu-pf27-dispatch-";
    int len = 1;
    for (int i = 0; prefix[i]; ++i) address.name[len++] = prefix[i];
    long parent = call(110, 0, 0, 0);
    char digits[20];
    int count = 0;
    do { digits[count++] = '0' + parent % 10; parent /= 10; } while (parent);
    while (count) address.name[len++] = digits[--count];
    address.name[len++] = argv[2][0];
    struct pollfd peers[3];
    for (int i = 0; i < 3; ++i) {
        long fd = call(41, 1, 1, 0);
        if (fd < 0 || call(42, fd, (long)&address, 2 + len)) die(91);
        peers[i] = (struct pollfd){(int)fd, 1, 0};
    }
    for (;;) {
        long ready = call(7, (long)peers, 3, 5000);
        if (ready == -4) continue;
        if (ready <= 0) die(93);
        if (peers[2].revents) {
            char command;
            if (call(0, peers[2].fd, (long)&command, 1) != 1) die(92);
            if (command == 'x') die(0);
            if (command != 'p') die(95);
            peers[0].events = 0;
            send_all(peers[2].fd, "k", 1);
        }
        for (int i = 0; i < 2; ++i) {
            if (!(peers[i].revents & 1) || !peers[i].events) {
                if (peers[i].revents & (8 | 16 | 32)) die(0);
                continue;
            }
            char bytes[8192];
            long count = call(0, peers[i].fd, (long)bytes, sizeof bytes);
            if (count == -4) continue;
            if (count <= 0) die(0);
            send_all(peers[1-i].fd, bytes, count);
        }
    }
}
__asm__(".global _start\n_start:\nmov %rsp,%rdi\nand $-16,%rsp\ncall start\nud2\n");
