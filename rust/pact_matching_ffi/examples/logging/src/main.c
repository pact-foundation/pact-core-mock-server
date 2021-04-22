
#include "pact_matching.h"
#include <stdlib.h>
#include <stdio.h>

#define ERROR_MSG_LEN 256

int main(void) {
    int status = 0;

    /*=======================================================================
     * Begin logger setup.
     *---------------------------------------------------------------------*/

    logger_init();
    
    /*=======================================================================
     * Attach a sink pointing info-level output to stdout.
     *---------------------------------------------------------------------*/

    status = logger_attach_sink("stdout", LevelFilter_Info);
    if (status != 0) {
        char error_msg[ERROR_MSG_LEN];
        int error = get_error_message(error_msg, ERROR_MSG_LEN);
        printf("%s\n", error_msg);
        return EXIT_FAILURE;
    }

    /*=======================================================================
     * Attach another sink pointing debug output to a log file.
     *---------------------------------------------------------------------*/

    status = logger_attach_sink("file /var/log/pm_ffi.log", LevelFilter_Debug);
    if (status != 0) {
        char error_msg[ERROR_MSG_LEN];
        int error = get_error_message(error_msg, ERROR_MSG_LEN);
        printf("%s\n", error_msg);
        return EXIT_FAILURE;
    }

    /*=======================================================================
     * Apply the logger, completing logging setup.
     *---------------------------------------------------------------------*/

    status = logger_apply();
    if (status != 0) {
        char error_msg[ERROR_MSG_LEN];
        int error = get_error_message(error_msg, ERROR_MSG_LEN);
        printf("%s\n", error_msg);
        return EXIT_FAILURE;
    }

    return EXIT_SUCCESS;
}

