# Correlation color-scale semantics

Publication config v1 supports explicit, renderer-neutral mappings for signed
correlation values in [-1, 1]. A scale is configured as optional
`colors.scale`; when omitted, the legacy negative/neutral/positive colors
are converted to a three-stop continuous scale. The existing
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

## Sequential palettes

A one-direction sequential palette is generally inappropriate for signed
correlations because negative and positive values have different directions
and zero is a meaningful neutral point. This contract therefore requires a
symmetric domain and an explicit zero stop/break rather than accepting a
misleading sequential mapping. Sequential palettes belong to non-signed
quantities and are outside this correlation-scale contract.

## Missing values and legends

Missing values are not zero and always use the separate `colors.missing`
color. Scene adaptation maps every matrix value to a color and produces
ordered legend entries for stops or intervals, followed by a distinct
`Missing` entry. The publication SVG renderer draws these entries below the
plot and expands its logical viewBox to keep the legend inside the output.

Colors are visual encodings only; the implementation does not infer
statistical meaning from arbitrary user-selected colors.
