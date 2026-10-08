Effects and Animation
=====================

Animation
---------

Animatable properties support keyframes, continuous and discrete keyframe
graphs, and Rhai expressions. Use expressions when a value should be derived
from project state instead of authored as individual keyframes. See
:doc:`expressions` for the available values, functions, and examples.

Visual effects
--------------

Visual modifiers cover transforms, opacity, repetition, paths, text masks,
color correction, blur, distortion, stylization, keys, and masks. Shrimply
also includes model-backed segmentation and transparent-fill operations.

Modifiers are applied in ordered stages so 3D, vector, rasterization, and
raster operations can be composed without silently changing their data type.

For fast-cut edits, **Shake** moves, rotates, and scales a visual about the
canvas center along smooth noise; keyframe its amplitude to hit beats.
**Flash** raises exposure in stops and mixes toward a color. **Glow** sums
several blur levels and can be tinted, **Chromatic aberration** has linear and
radial modes, and **Noise** can pick a new grain pattern several times per
second.

Retiming
--------

A video clip's **Speed ramp** multiplies its speed and can be keyframed: the
source advances by the area under the curve, so eased keyframes produce smooth
speed ramps. Expressions on the speed ramp are ignored. The clip **Frame rate**
holds each source frame for one frame of the chosen rate, like posterizing
time.

Clip transitions
----------------

**Mirror Mosaic** multiplies the frame into a grid of mirrored tiles, switches
clips at the densest point, and returns to a single frame. **Scale Transform**
zooms the outgoing clip toward the center and continues the same zoom into the
incoming clip, filling the edges with mirrored tiles. Both use the transition
curve, center, direction, and per-axis mirroring.

Audio effects
-------------

Audio modifiers include gain, pan, pitch, denoise, equalization, filters,
noise gates, stereo width, tremolo, bit crushing, chorus, compression,
limiting, reverb, echo, distortion, voice change, and microphone-style
coloration.

3D scenes
---------

3D scene content can include shapes, text, imported objects, a ground plane,
point lights, and sun lights. Imported OBJ, GLB, and PLY content can be used in
the same scene pipeline.
