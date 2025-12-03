#ifndef game_engine_h
#define game_engine_h

#include <stdint.h>

// Opaque handle to game state
typedef void* GameHandle;

// Initialize the game engine
// Returns a handle to use with other functions
GameHandle game_init(uint32_t width, uint32_t height);

// Handle surface resize
void game_resize(GameHandle handle, uint32_t width, uint32_t height);

// Update game state (call each frame before render)
void game_update(GameHandle handle);

// Render the game
void game_render(GameHandle handle);

// Set movement direction (0=none, 1=up, 2=down, 3=left, 4=right)
void game_set_direction(GameHandle handle, int32_t direction);

// Set game mode (0=manual, 1=auto)
void game_set_mode(GameHandle handle, int32_t mode);

// Handle touch events (action: 0=down, 1=up, 2=move)
void game_touch(GameHandle handle, float x, float y, int32_t action);

// Clean up and destroy the game engine
void game_destroy(GameHandle handle);

#endif /* game_engine_h */
