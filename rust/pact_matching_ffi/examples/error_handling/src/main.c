
#include "pact_matching.h"
#include <stdlib.h>
#include <stdio.h>

#define ERROR_MSG_LEN 256

int main(void) {
    /*=======================================================================
     * Simple empty message creation.
     *---------------------------------------------------------------------*/

    Message *msg = message_new();

    if (msg == NULL) {
        char error_msg[ERROR_MSG_LEN];
        int error = get_error_message(error_msg, ERROR_MSG_LEN);
        printf("%s\n", error_msg);
        return EXIT_FAILURE;
    }

    message_delete(msg);


    /*=======================================================================
     * Creating a message from a JSON string.
     *---------------------------------------------------------------------*/

    char *json_str = "{\
        \"description\": \"String\",\
        \"providerState\": \"provider state\",\
        \"matchingRules\": {}\
    }";
    Message *msg_json = message_new_from_json(0, json_str, PactSpecification_V3);

    if (msg_json == NULL) {
        char error_msg[ERROR_MSG_LEN];
        int error = get_error_message(error_msg, ERROR_MSG_LEN);
        printf("%s\n", error_msg);
        return EXIT_FAILURE;
    }

    message_delete(msg_json);

    return EXIT_SUCCESS;
}

