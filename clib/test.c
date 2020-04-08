#include "dp3t.h"
#include <stdio.h>

#define NUM_EPHEMS 8

int main() {
    dp3t_ephemeral eph[NUM_EPHEMS];
    const char filename[] = "session.json";
    
    dp3t_session session = dp3t_load_session(filename);
    if (session == 0) {
        session = dp3t_new_session();
        dp3t_save_session(session,filename);
    }
    
    dp3t_get_ephemerals(session, eph, NUM_EPHEMS);
    for (int i=0; i<NUM_EPHEMS; i++) {
        printf("%d) ", i);
        for (int j=0; j<16; j++) {
            printf("%02x", eph[i].token[j]);
        }
        printf("\n");
    }
    printf("\n");

    dp3t_session_key sk;
    dp3t_get_session_key(session,&sk);
    printf("JD: %d ", sk.julian_day);
    for (int i=0; i<32; i++) {
        printf("%02x", sk.key[i]);
    }
    printf("\n\n");

    for (int i=0; i<32; i++) {
        sk.key[i] = 0;
    }
    dp3t_replay_key replay = dp3t_new_replay(&sk, NUM_EPHEMS);
    replay_ephem ephem;
    while (dp3t_next(replay,&ephem)) {
        printf("repl JD: %d ", ephem.julian_day);
        for (int i=0; i<16; i++) {
            printf("%02x", ephem.ephem[i]);
        }
        printf("\n");
    }
    
    dp3t_free_replay(replay);
    dp3t_free_session(session);
}
