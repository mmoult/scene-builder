# Scene language description

The scene language is a subset of YAML format. Each scene may define objects and instances to describe the world.

Table of Contents:
- [Object](#object)
    * [Strip](#strip)
    * [Ray](#ray)
    * [Instance](#instance)
    * [Custom](#custom)
- [References](#references)
    * [Keywords](#keywords)
- [World](#world)

## Object

An object describes a blueprint for a three-dimensional geometry within world-space. There are two primitive objects
(`strip` and `ray`), which may be used directly or combined (via `instance` for transformational construction or
`data` for custom types) into more complex types.

Objects may define fields in additoin to the mandatory ones (excepting object `strip`, which is defined as a sequence,
and therefore cannot have any direct fields). These fields may use custom names which may not be used by some or any
export targets, but when a field is unsupported, it should be safely ignored.

Color is a notable optional field, which `strip` uses it for its faces (both front and back faces presently share) and
`ray` may use for its arrow.

### Strip

A `strip` (short for [Triangle strip](https://en.wikipedia.org/wiki/Triangle_strip)) is a sequence of *at least* three
3-dimensional vertices. For example, the simplest `strip` is a triangle:

```
strip
- [0, 1, 2]
- [3, 4, 5]
- [6, 7, 8]
```

To avoid data duplication, more vertices may be defined within the same strip. Here is an example of a quadrilateral:

```
strip
- (0, 0, 0)
- (1, 1, 1)
- (2, 0, 1)
- (1, -1, 2)
```

In the `strip`, a triangle is formed for each set of three adjacent vertices. Consider the four vertices of the
above-defined strip, now with names for each vertex:

```
A: (0, 0, 0)
B: (1, 1, 1)
C: (2, 0, 1)
D: (1, -1, 2)
```

The aforementioned `strip` forms two triangles: consisting of points ABC and BCD. Naturally, if the `strip` was extended
to contain new points, E, F, and G, then there would be 5 total triangles:

- ABC
- BCD
- CDE
- DEF
- EFG

Thus, the number of triangles = number of vertices - 2.

With more than three vertices, the ordering may affect the contour of the resultant geometry. Again, referencing the
previous example, the line segment from A to C runs along the geometry's surface. Contrast that with the line segment
from A to D, which does *not*.

Depending on the context, it may also be necessary to consider "winding order", which determines for each triangle which
face is the front and which is the back. Typically, the right-hand rule is applied, which is also the direction used by
a cross-product. To maintain a consistent front-face across a single triangle strip, the first two vertices of every
other triangle in a strip are defined in inverted-listing order. Therefore, we would see:

- triangle(ABC)
- triangle(CBD) <-- inverted BC
- triangle(CDE)
- triangle(EDF) <-- inverted DE
- triangle(EFG)

The pattern continues as expected if more vertices are used in a single triangle strip.

### Ray

In purely mathematical terms, a ray is geometry defined by an origin point and a direction. However, since the scene
language is used within the context of ray tracing, each `ray` has an extent, which may alternatively be thought of as a
parametric domain maximum. (Technically, the extent makes a `ray` into a line segment.)

Therefore, a `ray` is defined as a mapping with at least three mandatory fields: `origin`, `direction`, and `extent`.
Here is an example of a simple ray:

```
ray:
  origin: [-4.3, 2.8, -9.6]
  direction: [1, 1, 0]
  extent: 10
```

This defines a line segment from (-4.3, 2.8, -9.6) to (5.7, 12.8, -9.6).

If the direction is normalized, then the length of the line segment the `ray` forms is equal to its `extent`. However,
there is no requirement in the language for the direction to be normalized (that choice is left to the user).

### Instance

An instance is another object which has been transformed by some scaling, rotation, and/or translation. The only
mandatory field in the mapping is `instance`, which points to the object modified. The fields `scale`, `rotate`, and
`translate` are notable, but are optional. If they are present, they are applied in the order:

1. scale
2. rotate
3. translate

(even if they appear in some other order in the input file).

Scale is a 3D vector multiplier to the original size of the `instance` object. The value `1.0` represents no change in
that component, and therefore, `0.5` is half-sized and `2` is double sized.

Rotation is a 3D vector which describes, in degrees, the rotation of each component. Thus, `0` is no rotation and `180`
is rotation by pi around the origin. Rotations, if any, are applied sequentially, ie x, y, then z.

Translation is a 3D vector which describes repositioning of the instance in 3D space. Therefore, `0` indicates no
movement, `1` is offset by one for that component, and `-1` is offset by negative one for that component.

Below is an example instance:

```
instance: strip:
  - [0, 0, 0]
  - [1, 0, 0]
  - [0.5, 0, 0.5]
scale: [2.8, 0.4, 0]
rotate: [45, 0, 90]
translate: [3, -7, -5.2]
```

### Custom

Custom composite objects can be made by combining primitive objects (`strip` and `ray`) with each other and/or other
composite types. A custom object is defined as a mapping which doesn't contain the `instance` field or a mapping which
contains the `data` field, which itself is a sequence of other objects. Note: by this definition, a mapping which
contains *both* `data` and `instance` fields is treated as a custom object.

Fields defined in a custom object are recursively applied to all objects in its `data` (where recursive layers may
override any or all inherited fields with a more local definition). This follows the algorithm for reference resolution.
Compare with [references](#references).

Here is a complex custom object, demonstrating several custom fields:

```
aura: 3.2
color: [255, 0, 0]
data:
- strip:
  - [1, -1, 2]
  - [0.5, 1, 1]
  - [-1, -0.5, 1.5]
- ray:
    origin: [1, 2, 3]
    direction: [0, 0.5, 1.0]
    extent: 6
wireframe: true
```

## References
Any time a value appears in any object, a reference may be substituted instead (provided that the type of the reference
matches the type expected at use). This is valuable for reducing redundancy.

```
x_pos: 3.14159
my-tri: strip:
- [x_pos, 0, 0]
- [x_pos, 1.0, 2.0]
- [-3, 0.5, 3.0]
```

References are especially useful when using instances to transform the same object in several different ways:

```
- instance: my-tri
  translate: [0, 1, 2]
- instance: my-tri
  scale: [1, 2, 1]
  rotate: [0, 120, 60]
```

The reference is resolved by searching up the hierarchy for the most local definition matching the reference's name.
Consider the following example:

```
foo: [255, 0, 255]
bar:
  alpha:
    color: foo
  omega:
    foo: [0, 255, 255]
    beta:
      color: foo
baz:
  color: foo
```

In the example, the object `beta` inherits the teal (0, 255, 255) color since the `foo` definition in its parent,
`omega`, overrides the the definition of `foo` at the root. On the other hand, `baz` and `alpha` both get the magenta
color from the root-level `foo`. In summary:

| object  | value of `foo` | value of `color` |
|---------|----------------|------------------|
| `bar`   | (255, 0, 255)  | N/A              |
| `alpha` | (255, 0, 255)  | (255, 0, 255)    |
| `omega` | (0, 255, 255)  | N/A              |
| `beta`  | (0, 255, 255)  | (0, 255, 255)    |
| `baz`   | (255, 0, 255)  | (255, 0, 255)    |

### Keywords

When defining custom values, refrain from using the following keywords (ordered alphabetically):

- `data`
- `false`
- `instance`
- `ray`
- `strip`
- `true`

## World

Each scene file must have a `data` sequence at the document root, which describes the objects in the world. The world
sequence is a list of objects which must be rendered. In other words, any object defined outside the scope of world will
*not* be used unless there exists a reference to them within world's list (either directly or by recursive reference).

In this way, the scene root is a [custom object](#custom), with elevated importance and with the added qualification
that `data` must be a present field.
