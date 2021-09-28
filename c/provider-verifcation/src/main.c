#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pact.h>

int main (int argc, char **argv) {
  pactffi_log_to_buffer(LevelFilter_Trace);

  VerifierHandle *handle = pactffi_verifier_new();
  pactffi_verifier_set_provider_info(handle, "c-provider", NULL, NULL, 0, NULL);
  pactffi_verifier_add_file_source(handle, "pact.json");

  int result = pactffi_verifier_execute(handle);

  puts("--------------- LOGS ---------------");
  const char *logs = pactffi_verifier_logs(handle);
  printf("Got logs == %p\n", (void *) logs);
  printf("logs: %s\n", logs);
  puts("------------------------------------");

  pactffi_free_string(logs);
  pactffi_verifier_shutdown(handle);

  return result;
}
