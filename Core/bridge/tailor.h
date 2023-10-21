#ifndef TAILOR_H
#define TAILOR_H

#include <stdint.h>
#include <stdbool.h>

const uint32_t NEW_FILE_STARTED = 0; //< New file watch process started
const uint32_t NEW_LINES_ADDED  = 1; //< New lines appended to the file being watched

const int32_t INVALID_CLIENT_ID = -1;

//! Cerate instance of Tailor
//! @return Pointer to created instance
void* tailor_init();

//! Destroy Tailor instance
//! @param instance Pointer to Tailor instance
void tailor_destroy(void* instance);

//! Set callback to be called when new lines appear in log
//! @param instance Pointer to Tailor instance
//! @param callback Callback to be called
void tailor_set_new_lines_callback(void* instance, void (*callback)(int32_t client_id, uint32_t msg_type, uint32_t strings_count, const char** msg));

//! Start watching path
//! @param instance Pointer to Tailor instance
//! @param path Path to a file or directory to watch
//! @return Client ID, see new_lines_callback for details
int32_t tailor_watch_path(void* instance, const char* path);

//! Stop watching path for the client_id
//! @param client_id Client ID to stop watch
bool tailor_stop_watch(void* instance, int32_t client_id);

#endif //TAILOR_H
