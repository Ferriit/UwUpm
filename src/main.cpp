#include <cstdlib>
#include "main.h"
#include <sys/wait.h>

int Cpp_Command(const char* cmd) {
    int ret = system(cmd);
    if (WIFEXITED(ret)) {
        return WEXITSTATUS(ret);
    }
    else {
        return 1;
    }
}
