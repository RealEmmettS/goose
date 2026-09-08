#ifndef HONK_OWNED_PROPS_H
#define HONK_OWNED_PROPS_H
#include <stddef.h>
#include <stdint.h>

enum { HONK_PROP_NOTE, HONK_PROP_IMAGE, HONK_PROP_MOVE, HONK_PROP_PASSTHROUGH,
       HONK_PROP_FOCUS, HONK_PROP_TEXT, HONK_PROP_CLOSE, HONK_PROP_SHUTDOWN };
typedef struct {
    uint64_t id;
    uint32_t operation;
    int32_t x, y;
    uint32_t width, height, pixel_width, pixel_height;
    int32_t passthrough;
    const char *title;
    const char *text;
    const unsigned char *pixels;
    size_t pixels_len;
} HonkPropCommand;

int honk_props_run(const unsigned char *light, size_t light_len, const unsigned char *bold, size_t bold_len);
int honk_props_apply(const HonkPropCommand *command);
int honk_props_decode(const unsigned char *json, size_t length);
#endif
