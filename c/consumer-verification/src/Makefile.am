bin_PROGRAMS = consumer-verification
consumer_verification_SOURCES = main.c
AM_CPPFLAGS = ${LIBCURL_CPPFLAGS} -I../../rust/pact_matching_ffi/include
consumer_verification_LDADD = ${LIBCURL} -ldl
