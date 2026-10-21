// keyflip C interface. One process-wide lexicon, one session per typing client.
//
// Requests and responses are the JSON documents of keyflip-proto
// (see crates/keyflip-proto/src/lib.rs): {"op":"key","key":{"code":"a",...}}.
// All strings are UTF-8 and NUL-terminated.

#ifndef KEYFLIP_H
#define KEYFLIP_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct KeyflipSession KeyflipSession;

// Creates a session. The first call loads the lexicon (about 100 ms).
KeyflipSession *keyflip_session_new(void);

// Destroys a session. Passing NULL is allowed.
void keyflip_session_free(KeyflipSession *session);

// Handles one request document and returns one response document.
// The result is never NULL for a valid session; free it with keyflip_string_free.
// Returns NULL only when session or request is NULL or the request is not UTF-8.
char *keyflip_handle(KeyflipSession *session, const char *request_json);

// Frees a string returned by keyflip_handle. Passing NULL is allowed.
void keyflip_string_free(char *s);

#ifdef __cplusplus
}
#endif

#endif
