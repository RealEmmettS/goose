#pragma once
#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct HonkAccessibility HonkAccessibility;
typedef struct {
    uint64_t id;
    int32_t action;
    size_t text_len;
    uint8_t text[4096];
} HonkAccessibilityAction;

HonkAccessibility *honk_a11y_create(void *window);
void honk_a11y_destroy(HonkAccessibility *bridge);
int honk_a11y_publish(HonkAccessibility *bridge, const uint8_t *json, size_t len, double scale);
int honk_a11y_poll(HonkAccessibility *bridge, HonkAccessibilityAction *output);
void honk_a11y_focus(HonkAccessibility *bridge, int focused);
#ifndef _WIN32
void honk_a11y_bounds(HonkAccessibility *bridge, double x, double y, double width, double height);
#endif
#ifdef _WIN32
int honk_a11y_getobject(HonkAccessibility *bridge, size_t wparam, intptr_t lparam, intptr_t *result);
#endif

#ifdef __cplusplus
}
#endif
