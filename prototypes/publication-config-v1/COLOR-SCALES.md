# Correlation and general-matrix color-scale semantics

Publication config v1 supports renderer-neutral color mappings for both
correlation values in [-1, 1] and arbitrary finite general numeric matrices.
A scale is configured as optional `colors.scale`; when omitted, correlation
and mixed-sign matrices use the signed diverging scale, while one-sided
general matrices use the legacy `YlOrBr` sequential palette. The existing
`colors.missing` remains the missing-value color.

## Continuous diverging scales

A continuous scale uses ordered `stops`, each containing a numeric
`value` and six-digit sRGB hex `color`. Stops cover both endpoints (-1 and
1), include zero, and are symmetric around zero. The supported interpolation
is channel-wise sRGB interpolation between adjacent stops.

```json
{
  "scale": {
    "type": "continuous",
    "interpolation": "rgb",
    "stops": [
      {"value": -1, "color": "#2166ac"},
      {"value": 0, "color": "#f7f7f7"},
      {"value": 1, "color": "#b2182b"}
    ]
  }
}
```

Intermediate stops are supported, but their positions must remain symmetric
around zero. Colors themselves need not be mirror images: they are visual
choices, not statistical assertions.

## Discrete scales

A discrete scale uses strictly increasing `breaks` covering [-1, 1],
symmetric around zero, with zero as an explicit boundary, and exactly one
color per interval.

```json
{
  "scale": {
    "type": "discrete",
    "breaks": [-1, -0.5, 0, 0.5, 1],
    "colors": ["#053061", "#4393c3", "#d6604d", "#67001f"]
  }
}
```

Intervals are left-closed and right-open; only the final interval includes
its upper endpoint. Thus a value exactly on a break belongs to the interval
starting at that break. Legend labels use the same convention.

## Sequential scales for general numeric matrices

A sequential scale uses ordered stops covering the normalized visual domain
[0, 1]. General numeric matrices whose observed values are all non-negative
or all non-positive use the legacy `YlOrBr` palette by default. Values are
normalized monotonically from the observed minimum to maximum, so negative-only
matrices retain their numeric ordering while their original signed values
remain unchanged in the scene. A constant range is padded deterministically
before normalization. An explicit sequential scale can replace the default
palette:

```json
{
  "scale": {
    "type": "sequential",
    "interpolation": "rgb",
    "stops": [
      {"value": 0, "color": "#ffffe5"},
      {"value": 0.5, "color": "#fec44f"},
      {"value": 1, "color": "#662506"}
    ]
  }
}
```

Mixed-sign general matrices use the legacy `RdBu` diverging palette by default.
The palette family is selected from observed values, not from an expanded
explicit color domain. Explicit limits must cover all observed values; values
outside them are rejected rather than silently clipped.

## Missing values and legends

Missing values are not zero and always use the separate `colors.missing`
color. Scene adaptation maps every matrix value to a color and produces
ordered legend entries for stops or intervals, followed by a distinct
`Missing` entry. The publication SVG renderer draws these entries below the
plot and expands its logical viewBox to keep the legend inside the output.

Colors are visual encodings only; the implementation does not infer
statistical meaning from arbitrary user-selected colors.
