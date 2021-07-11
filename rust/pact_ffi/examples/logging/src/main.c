
#include "pact.h"
#include <stdlib.h>
#include <stdio.h>

#define ERROR_MSG_LEN 256

int main(void) {
    int status = 0;

    /*=======================================================================
     * Begin logger setup.
     *---------------------------------------------------------------------*/

    pactffi_logger_init();
    
    /*=======================================================================
     * Attach a sink pointing info-level output to stdout.
     *---------------------------------------------------------------------*/

    status = pactffi_logger_attach_sink("stdout", LevelFilter_Info);
    if (status != 0) {
        char error_msg[ERROR_MSG_LEN];
        int error = pactffi_get_error_message(error_msg, ERROR_MSG_LEN);
        printf("%s\n", error_msg);
        return EXIT_FAILURE;
    }

    /*=======================================================================
     * Attach another sink pointing debug output to a log file.
     *---------------------------------------------------------------------*/

    status = pactffi_logger_attach_sink("file ./pm_ffi.log", LevelFilter_Debug);
    if (status != 0) {
        char error_msg[ERROR_MSG_LEN];
        int error = pactffi_get_error_message(error_msg, ERROR_MSG_LEN);
        printf("%s\n", error_msg);
        return EXIT_FAILURE;
    }

    /*=======================================================================
     * Apply the logger, completing logging setup.
     *---------------------------------------------------------------------*/

    status = pactffi_logger_apply();
    if (status != 0) {
        char error_msg[ERROR_MSG_LEN];
        int error = pactffi_get_error_message(error_msg, ERROR_MSG_LEN);
        printf("%s\n", error_msg);
        return EXIT_FAILURE;
    }

    return EXIT_SUCCESS;
}
