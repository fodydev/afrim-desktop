/*
 * AfrimDesktop IME - C FFI Header
 *
 * This header defines the C interface to the AfrimDesktop IME engine.
 * Use this to integrate AfrimDesktop with fcitx5 or other input method frameworks.
 */

#ifndef AFRIM_DESKTOP_H
#define AFRIM_DESKTOP_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque handle to a AfrimDesktop engine instance */
typedef struct AfrimDesktopEngine AfrimDesktopEngine;

/*
 * Create a new AfrimDesktop engine instance.
 * Returns a pointer to the engine, or NULL on failure.
 * The caller is responsible for freeing the engine with afrim_desktop_engine_free().
 */
AfrimDesktopEngine* afrim_desktop_engine_new(void);

/*
 * Load the afrim dataset selected by the user settings ($AFRIM_CONFIG,
 * [afrim] config_path, or the built-in sample).
 * Returns 0 on success, -1 on failure (the built-in sample stays active).
 */
int afrim_desktop_engine_init(AfrimDesktopEngine* engine);

/*
 * Destroy a AfrimDesktop engine instance and free its resources.
 */
void afrim_desktop_engine_free(AfrimDesktopEngine* engine);

/*
 * Process a key event.
 *
 * Parameters:
 *   engine     - The engine instance
 *   keysym     - X11 keysym value
 *   state      - Modifier state (X11 modifier mask)
 *   is_release - 1 if key release, 0 if key press
 *
 * Returns 1 if the key was consumed by the IME, 0 otherwise.
 *
 * After calling this function, check the has_* functions to see what
 * actions need to be taken (update preedit, commit text, etc.)
 */
int afrim_desktop_engine_process_key(
    AfrimDesktopEngine* engine,
    uint32_t keysym,
    uint32_t state,
    int is_release
);

/*
 * Reset the engine state, clearing any pending input.
 */
void afrim_desktop_engine_reset(AfrimDesktopEngine* engine);

/*
 * Commit the candidate at `index` (0-based) on the current candidate page.
 * Returns 1 if a candidate was committed, 0 otherwise.
 * After calling this function, check the has_* functions as for process_key.
 */
int afrim_desktop_engine_select_candidate(AfrimDesktopEngine* engine, uint32_t index);

/*
 * Set the surrounding text context from the editor.
 * No-op kept for ABI compatibility: afrim does not use surrounding text.
 */
void afrim_desktop_engine_set_surrounding_text(
    AfrimDesktopEngine* engine,
    const char* text,
    uint32_t cursor_pos
);

/* --- Preedit (composition) text --- */

/*
 * Check if there's a preedit update pending.
 */
int afrim_desktop_engine_has_preedit(const AfrimDesktopEngine* engine);

/*
 * Get the current preedit text.
 * Returns a pointer to a null-terminated UTF-8 string.
 * The pointer is valid until the next process_key call.
 */
const char* afrim_desktop_engine_get_preedit(const AfrimDesktopEngine* engine);

/*
 * Get the preedit text length in bytes (not including null terminator).
 */
uint32_t afrim_desktop_engine_get_preedit_len(const AfrimDesktopEngine* engine);

/*
 * Get the preedit caret (cursor) position in bytes.
 * This indicates where the cursor should be displayed within the preedit text.
 */
uint32_t afrim_desktop_engine_get_preedit_caret(const AfrimDesktopEngine* engine);

/* --- Commit text --- */

/*
 * Check if there's a commit pending.
 */
int afrim_desktop_engine_has_commit(const AfrimDesktopEngine* engine);

/*
 * Get the commit text.
 * Returns a pointer to a null-terminated UTF-8 string.
 * The pointer is valid until the next process_key call.
 */
const char* afrim_desktop_engine_get_commit(const AfrimDesktopEngine* engine);

/*
 * Get the commit text length in bytes (not including null terminator).
 */
uint32_t afrim_desktop_engine_get_commit_len(const AfrimDesktopEngine* engine);

/* --- Candidates --- */

/*
 * Check if there's a candidates update pending.
 */
int afrim_desktop_engine_has_candidates(const AfrimDesktopEngine* engine);

/*
 * Check if candidates should be hidden.
 */
int afrim_desktop_engine_should_hide_candidates(const AfrimDesktopEngine* engine);

/*
 * Get the number of candidates.
 */
uint32_t afrim_desktop_engine_get_candidate_count(const AfrimDesktopEngine* engine);

/*
 * Get a candidate by index.
 * Returns a pointer to a null-terminated UTF-8 string, or NULL if index is out of range.
 * The pointer is valid until the next process_key call.
 */
const char* afrim_desktop_engine_get_candidate(const AfrimDesktopEngine* engine, uint32_t index);

/*
 * Get a candidate annotation (comment) by index.
 * Returns a pointer to a null-terminated UTF-8 string (e.g. "🤖", "📚"),
 * or NULL if index is out of range. Empty string means no annotation.
 * The pointer is valid until the next process_key call.
 */
const char* afrim_desktop_engine_get_candidate_annotation(const AfrimDesktopEngine* engine, uint32_t index);

/*
 * Get the current candidate cursor position (selected index).
 */
uint32_t afrim_desktop_engine_get_candidate_cursor(const AfrimDesktopEngine* engine);

/* --- Auxiliary text (reading hint) --- */

/*
 * Check if there's an aux text update pending.
 */
int afrim_desktop_engine_has_aux(const AfrimDesktopEngine* engine);

/*
 * Get the aux text.
 * Returns a pointer to a null-terminated UTF-8 string.
 * The pointer is valid until the next process_key call.
 */
const char* afrim_desktop_engine_get_aux(const AfrimDesktopEngine* engine);

/*
 * Get the aux text length in bytes (not including null terminator).
 */
uint32_t afrim_desktop_engine_get_aux_len(const AfrimDesktopEngine* engine);

/* --- Timing --- */

/*
 * Get the last conversion time in milliseconds (inference only).
 */
uint64_t afrim_desktop_engine_get_last_conversion_ms(const AfrimDesktopEngine* engine);

/*
 * Get the last process_key time in milliseconds (input to result, end-to-end).
 */
uint64_t afrim_desktop_engine_get_last_process_key_ms(const AfrimDesktopEngine* engine);

/* --- Learning cache --- */

/*
 * No-op kept for ABI compatibility: the afrim engine has no learning cache.
 */
void afrim_desktop_engine_save_learning(AfrimDesktopEngine* engine);

/* --- State query --- */

/*
 * Check if the engine is in the Empty (idle) state.
 * Returns 1 if empty, 0 if composing or converting.
 */
int afrim_desktop_engine_is_empty(const AfrimDesktopEngine* engine);

/* --- Focus handling --- */

/*
 * Commit any pending input.
 * This is used when the IME is deactivated (focus lost) to commit preedit.
 * Returns 1 if text was committed, 0 otherwise.
 * After this call, check afrim_desktop_engine_has_commit() and get the text
 * with afrim_desktop_engine_get_commit().
 */
int afrim_desktop_engine_commit(AfrimDesktopEngine* engine);

#ifdef __cplusplus
}
#endif

#endif /* AFRIM_DESKTOP_H */
