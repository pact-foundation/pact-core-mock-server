
#include "pact_matching.h"
#include <stdlib.h>
#include <stdio.h>

#define ERROR_MSG_LEN 256

int main(void) {
    logger_init();
    logger_attach_sink("stdout", LevelFilter_Trace);
    logger_apply();

    Message *msg = message_new();
    int error = message_delete(msg);

    if (error == EXIT_FAILURE) {
        char error_msg[ERROR_MSG_LEN];

        int error = get_error_message(error_msg, ERROR_MSG_LEN);

        printf("%s\n", error_msg);

        return EXIT_FAILURE;
    }

    char *json_str = "{\
        \"description\": \"String\",\
        \"providerState\": \"provider state\",\
        \"matchingRules\": {}\
    }";
    Message *msg_json = message_from_json(0, json_str, PactSpecification_V3);

    if (NULL == msg_json) {
        char error_msg[ERROR_MSG_LEN];
        int error = get_error_message(error_msg, ERROR_MSG_LEN);
        printf("%s\n", error_msg);
        return EXIT_FAILURE;
    }

    return EXIT_SUCCESS;
}
