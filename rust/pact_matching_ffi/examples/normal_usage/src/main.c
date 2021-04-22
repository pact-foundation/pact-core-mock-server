
#include "pact_matching.h"
#include <stdlib.h>
#include <stdio.h>

#define ERROR_MSG_LEN 256

#define NULLCHK(ptr) {\
    char msg[ERROR_MSG_LEN];\
    int error = get_error_message(msg, ERROR_MSG_LEN);\
    if (error != 0) exit(EXIT_FAILURE);\
    printf("%s\n", msg);\
    exit(EXIT_FAILURE);\
}

Message* msg_json() {
    const int id = 0;
    const int spec = PactSpecification_V3;
    const char *json = "{\
        \"description\": \"A basic message.\",\
        \"providerStates\": [\
	    { \"name\": \"state 1\", \"params\": {} },\
	    { \"name\": \"state 2\", \"params\": {} },\
	    { \"name\": \"state 3\", \"params\": {} }\
        ]\
    }";

    Message *msg = message_new_from_json(id, json, spec);
    NULLCHK(msg);

    return msg;
}

int main(void) {
    Message *msg = msg_json();

    ProviderStateIterator *iter = message_get_provider_state_iter(msg);

    while (iter != NULL) {
        ProviderState *state = provider_state_iter_next(iter);
	const char *name = provider_state_get_name(state);
	printf("Name: %s\n", name);
    }

    provider_state_iter_delete(iter);
    message_delete(msg);

    return EXIT_SUCCESS;
}

