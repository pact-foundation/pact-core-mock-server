
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
     * Attach another sink to collect log events into a memory buffer.
     *---------------------------------------------------------------------*/

    status = pactffi_logger_attach_sink("buffer", LevelFilter_Trace);
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

    pactffi_log_message("example C", "debug", "This is a debug message");
    pactffi_log_message("example C", "info", "This is an info message");
    pactffi_log_message("example C", "error", "This is an error message");
    pactffi_log_message("example C", "trace", "This is a trace message");

    char *logs = pactffi_fetch_log_buffer(NULL);
    if (logs == NULL) {
        printf("Could not get the buffered logs\n");
        return EXIT_FAILURE;
    }

    printf("---- Logs from buffer ----\n");
    printf("%s", logs);
    printf("--------------------------\n");
    pactffi_string_delete(logs);

    return EXIT_SUCCESS;
}
