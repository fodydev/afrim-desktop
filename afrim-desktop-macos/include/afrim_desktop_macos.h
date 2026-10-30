#ifndef AFRIM_DESKTOP_MACOS_H
#define AFRIM_DESKTOP_MACOS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque engine handle */
typedef struct AfrimDesktopMacEngine AfrimDesktopMacEngine;

/* Lifecycle */
AfrimDesktopMacEngine *afrim_desktop_macos_engine_new(void);
int afrim_desktop_macos_engine_init(AfrimDesktopMacEngine *engine);
void afrim_desktop_macos_engine_free(AfrimDesktopMacEngine *engine);

/* Input */
int afrim_desktop_macos_process_key(
    AfrimDesktopMacEngine *engine,
    uint16_t keycode,
    uint32_t unicode_char,
    int has_unicode,
    int shift,
    int ctrl,
    int opt,
    int cmd,
    int is_press
);
void afrim_desktop_macos_reset(AfrimDesktopMacEngine *engine);
int afrim_desktop_macos_commit(AfrimDesktopMacEngine *engine);
/* Commit the candidate at index (0-based) on the current page; 1 if committed */
int afrim_desktop_macos_select_candidate(AfrimDesktopMacEngine *engine, unsigned int index);
/* No-ops kept for ABI compatibility (afrim has no learning cache / context) */
void afrim_desktop_macos_save_learning(AfrimDesktopMacEngine *engine);
void afrim_desktop_macos_set_surrounding_text(
    AfrimDesktopMacEngine *engine,
    const char *text,
    unsigned int cursor_pos
);

/* Preedit queries */
int afrim_desktop_macos_has_preedit(const AfrimDesktopMacEngine *engine);
const char *afrim_desktop_macos_get_preedit(const AfrimDesktopMacEngine *engine);
unsigned int afrim_desktop_macos_get_preedit_len(const AfrimDesktopMacEngine *engine);
unsigned int afrim_desktop_macos_get_preedit_caret(const AfrimDesktopMacEngine *engine);

/* Commit queries */
int afrim_desktop_macos_has_commit(const AfrimDesktopMacEngine *engine);
const char *afrim_desktop_macos_get_commit(const AfrimDesktopMacEngine *engine);
unsigned int afrim_desktop_macos_get_commit_len(const AfrimDesktopMacEngine *engine);

/* Candidate queries */
int afrim_desktop_macos_has_candidates(const AfrimDesktopMacEngine *engine);
int afrim_desktop_macos_should_hide_candidates(const AfrimDesktopMacEngine *engine);
unsigned int afrim_desktop_macos_get_candidate_count(const AfrimDesktopMacEngine *engine);
const char *afrim_desktop_macos_get_candidate(const AfrimDesktopMacEngine *engine, unsigned int index);
const char *afrim_desktop_macos_get_candidate_annotation(const AfrimDesktopMacEngine *engine, unsigned int index);
unsigned int afrim_desktop_macos_get_candidate_cursor(const AfrimDesktopMacEngine *engine);

/* Aux text queries */
int afrim_desktop_macos_has_aux(const AfrimDesktopMacEngine *engine);
const char *afrim_desktop_macos_get_aux(const AfrimDesktopMacEngine *engine);

/* State */
/* 0 = Afrim, 1 = Direct input */
int afrim_desktop_macos_input_mode(const AfrimDesktopMacEngine *engine);

#ifdef __cplusplus
}
#endif

#endif /* AFRIM_DESKTOP_MACOS_H */
