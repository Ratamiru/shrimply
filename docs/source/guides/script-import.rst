Markdown Scripts
================

A Markdown script lays out a video as text. Open a ``.md`` file like a project:
Shrimply builds a timeline from it and asks where to save the new project.
Edit the script and import it again to rebuild the timeline; changes made to
an earlier imported project are not carried over.

.. code-block:: markdown

   ---
   size: 1920x1080
   fps: 30
   wpm: 150
   captions: true
   ---

   # Headings only organise the file

   ![](clip.mp4){at=12 dur=4 in=fade out=blur t=0.3}
   ![](photo.jpg){zoom=1..1.3 fit=contain}
   ![](music.mp3)
   > The narration for this beat.

Beats
-----

A beat is a run of media links followed by quote lines. The quote lines are the
narration: they become one caption and set how long the beat lasts, from the
word count and the reading speed. The next media link, a heading, or a ``---``
rule starts a new beat. Other prose and HTML comments are notes that never
reach the video.

Visual links play one after another. Links without ``dur`` share the time the
narration needs; when the narration is longer than the shots, the last shot is
held until it has been read. A beat without narration uses each clip's own
length, or three seconds for a still image. Audio links start with their beat
and stack on the first free audio track.

Settings
--------

.. list-table::
   :widths: 20 80
   :header-rows: 1

   * - Key
     - Meaning
   * - ``size``
     - Canvas size, ``1920x1080`` by default.
   * - ``fps``
     - Frame rate as a number or ratio such as ``30000/1001``; 30 by default.
   * - ``wpm``
     - Narration reading speed in words per minute; 150 by default.
   * - ``captions``
     - ``false`` leaves the narration out of the caption track.

Link attributes
---------------

.. list-table::
   :widths: 20 80
   :header-rows: 1

   * - Attribute
     - Meaning
   * - ``dur``
     - Length in seconds (``4``, ``2.5``, ``1/3``) or ``auto``.
   * - ``at``
     - Offset into the source in seconds.
   * - ``in``, ``out``
     - Intro and outro transition, such as ``fade``, ``slide``, ``zoom``,
       ``blur``, ``pixelate``, ``iris`` or ``dissolve``. Transitions that only
       animate text and shapes are rejected for media.
   * - ``t``
     - Transition length, 0.5 seconds by default. A transition longer than the
       shot, or than half of it when both ``in`` and ``out`` are set, is
       shortened with a warning.
   * - ``zoom``
     - Slow zoom across the shot, such as ``1..1.3``.
   * - ``fit``
     - ``cover`` (default), ``contain``, ``natural`` or ``stretch``.

Relative paths are resolved from the script's folder. Media is not copied into
the project.
