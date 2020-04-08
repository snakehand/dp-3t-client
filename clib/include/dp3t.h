#ifndef _DP3T_LIB_H_
#include <stdint.h>

typedef struct {
    uint8_t token[16];
} dp3t_ephemeral;

typedef struct {
    uint32_t julian_day;
    uint8_t key[32];
} dp3t_session_key;

typedef struct {
    uint32_t julian_day;
    uint8_t ephem[16];
} replay_ephem;

typedef void* dp3t_session;
typedef void* dp3t_replay_key;

// Create a new session
extern dp3t_session dp3t_new_session();

// Load an existing session
extern dp3t_session dp3t_load_session(const char*);

// Get ephemerals for today
extern void dp3t_get_ephemerals(dp3t_session, dp3t_ephemeral*, uint32_t num);

// Save session
extern int dp3t_save_session(dp3t_session, const char*);

// Retrieve session key
extern int dp3t_get_session_key(dp3t_session, dp3t_session_key*);

// Free the session
extern void dp3t_free_session(dp3t_session);

// Create a replay key
extern dp3t_replay_key dp3t_new_replay(dp3t_session_key*, uint32_t num);

// Get next key from replay
extern int dp3t_next(dp3t_replay_key, replay_ephem*);

// Free replay key
extern void dp3t_free_replay(dp3t_replay_key);


#endif
