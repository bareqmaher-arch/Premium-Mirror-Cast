// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Bareq Maher (Premium Cast changes)
// The Premium theme's backdrop (and the installer's): a fine signal grid of
// dots that spreads outward from the logo, crossed by faint broadcast rings,
// with a soft cast wave travelling outward every few seconds.
// Samples no textures. Output is premultiplied.
//
// Compiled to hive.frag.qsb with Qt's qsb (see shaders.qrc). The file keeps
// its old name so the build and the QML that load it stay unchanged.
#version 440

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    vec2 itemSize;
    vec2 origin;      // where it spreads from, in pixels
    float time;       // seconds
    float reveal;     // how far it has spread, in pixels
    float cell;       // grid pitch, in pixels
    float pulse;      // 0: still (reduced motion), 1: animated
    float strength;   // overall intensity
    float breathPeriod; // seconds per breath
    float pulsePeriod;  // seconds between cast waves
    float corner;       // rounded-rectangle mask radius in pixels (0: none)
    float reach;        // how far the grid carries (fraction of the diagonal)
    float pulseReach;   // how far a wave travels (fraction of the diagonal)
    vec4 lineColor;
    vec4 glowColor;
};

float hash(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

void main() {
    vec2 px = qt_TexCoord0 * itemSize;
    float d = distance(px, origin);
    float far = length(itemSize);

    // The dot grid: one small round dot per cell.
    vec2 g = (px - origin) / cell;
    vec2 id = floor(g);
    float dotDist = length(fract(g) - 0.5) * cell;   // pixels from the dot centre
    float dotMask = 1.0 - smoothstep(0.9, 1.7, dotDist);

    // Broadcast rings around the origin, every four cells, one pixel wide.
    float spacing = cell * 4.0;
    float ringDist = abs(fract(d / spacing + 0.5) - 0.5) * spacing;
    float rings = (1.0 - smoothstep(0.0, 1.1, ringDist)) * 0.35;

    // Spreads from the logo; the front is soft.
    float grown = 1.0 - smoothstep(reveal - 90.0, reveal, d);
    // Fades towards the corners so the content stays calm.
    float falloff = exp(-d / (far * reach));

    // Breathing: a slow rise and fall of the whole field.
    float breathe = 0.80 + 0.20 * sin(time * 6.2831853 / breathPeriod) * pulse;

    // A cast wave travelling outward every few seconds.
    float t = fract(time / pulsePeriod);
    float waveRadius = t * far * pulseReach;
    float wave = exp(-pow((d - waveRadius) / 60.0, 2.0)) * (1.0 - t) * pulse;

    // A little variation from dot to dot, like a live signal.
    float shade = 0.7 + 0.6 * hash(id);

    float lineA = (dotMask * shade + rings) * grown * falloff * breathe * 1.5
                + (dotMask + rings) * grown * wave * 1.9;
    float fillA = grown * wave * 0.05;
    float glowA = exp(-d / 170.0) * breathe * 0.30 * grown;

    vec3 rgb = lineColor.rgb * lineA * lineColor.a + glowColor.rgb * (glowA + fillA) * glowColor.a;
    float a = clamp(lineA * lineColor.a + (glowA + fillA) * glowColor.a, 0.0, 1.0);
    // Inside a rounded rectangle when asked (the phone preview's screen).
    float inside = 1.0;
    if (corner > 0.0) {
        vec2 q = abs(px - itemSize * 0.5) - (itemSize * 0.5 - vec2(corner));
        float sd = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - corner;
        inside = 1.0 - smoothstep(-0.75, 0.75, sd);
    }

    float k = strength * qt_Opacity * inside;
    fragColor = vec4(rgb * k, a * k);
}
