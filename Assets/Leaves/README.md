# Autumn leaf artwork

Four original vector designs provide distinct maple, oak, birch and ginkgo silhouettes.
The initial botanical artwork was generated for this project with Quiver, then inspected,
normalized and recolored to the existing autumn palette. No external images or fonts are used.

`masters/` contains the editable SVG geometry. `palette.json` defines the four existing colors.
Run `python script/build_leaf_art.py` to regenerate the sixteen SVG color variants and the
checked-in Rust paths. Use `--check` to detect drift. The runtime draws these paths into a
fixed, small supersampled image cache once; it needs no SVG parser, filesystem access or
network service to draw the leaves.

Color, leaf family, size, mirroring and rotation vary independently. This artwork does not
change the simulation's random draws, leaf counts, pile lifetime or charging scatter.
