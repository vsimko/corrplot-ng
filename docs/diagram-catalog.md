---
type: Feature Catalog
title: Catalog of diagrams and visualizations for corrplot-ng
description: A working list of existing plot methods, shared display options, and possible future visualizations.
tags:
  - visualization
  - diagrams
  - features
status: draft
---

# Catalog of diagrams and visualizations for corrplot-ng

## Purpose

This catalog lists the visual forms that corrplot-ng may support. It separates compatibility targets from ideas for later work.

This is a planning document. A listed item is not proof that the feature exists.

## 1. Existing correlation-matrix methods

The first goal is to cover the methods in the original R corrplot package.

The thumbnails below are small, schematic examples. They explain the visual form only. They do not define exact colors, scales, defaults, or output from the future implementation.

| Method | Example | Description | Plan |
|---|---|---|---|
| Circle | ![Circle method example](images/circle.svg) | Show each correlation as a circle. Use size and color to show its value. | Compatibility target |
| Square | ![Square method example](images/square.svg) | Show each correlation as a square. Use size and color to show its value. | Compatibility target |
| Ellipse | ![Ellipse method example](images/ellipse.svg) | Show each correlation as an ellipse. Use shape and color to show its value. | Compatibility target |
| Number | ![Number method example](images/number.svg) | Show each correlation as a number. | Compatibility target |
| Shade | ![Shade method example](images/shade.svg) | Use shaded cells to show correlations. | Compatibility target |
| Color | ![Color method example](images/color.svg) | Use colored cells to show correlations. | Compatibility target |
| Pie | ![Pie method example](images/pie.svg) | Show each correlation as a pie-like shape. | Compatibility target |

## 2. Matrix layouts

These are layout options. They are not separate plot types.

- **Full matrix:** show both triangles of the matrix.
- **Upper triangle:** show only the upper triangle.
- **Lower triangle:** show only the lower triangle.
- **Mixed triangles:** use different methods or settings in the upper and lower triangles.
- **Diagonal control:** show, hide, or change the diagonal cells.
- **Variable order:** use the input order or a supported ordering method.
- **Variable groups:** show groups of related variables when the data and method support them.

## 3. Labels and annotations

The project should assess these shared options:

- Variable names and axis labels.
- Correlation values.
- Significance markers.
- Confidence intervals, where the input method provides them.
- Legends and color scales.
- Text size and label rotation.
- Missing-value display.
- Notes for selected cells.

The exact behavior and defaults must be defined in the API design.

## 4. Possible new visualizations

These ideas need design and user feedback before implementation.

| Visualization | Description | Priority |
|---|---|---|
| Clustered correlation heatmap | Group variables by similarity and show the resulting matrix. | Candidate |
| Pairwise correlation panel | Show several pairwise relationships in a grid. | Candidate |
| Correlation network | Show variables as nodes and selected correlations as links. | Candidate |
| Interactive correlation matrix | Let users inspect values and select variables in a web view. | Candidate |
| Multi-matrix comparison | Compare correlation matrices from different datasets or time periods. | Candidate |
| Correlation change plot | Show how correlations change between groups or time periods. | Candidate |
| Partial-correlation view | Show partial correlations when suitable input values are available. | Candidate |

## 5. Shared visual rules

Where possible, plot types should use the same rules for:

- Mapping values to colors.
- Mapping values to size or shape.
- Handling missing values.
- Showing positive and negative values.
- Showing legends and scales.
- Selecting and ordering variables.
- Showing annotations.
- Exporting or rendering output.

A shared rule should have one definition in the core. Language interfaces should not change its meaning without a clear reason.

## 6. How to add an item

For each new item, record:

1. The user problem it solves.
2. The data it needs.
3. The visual form and its meaning.
4. The options and defaults.
5. How it handles missing or invalid data.
6. Whether it must match an existing R feature.
7. How to test it.
8. Which language interfaces should support it.

## 7. Next steps

- Check the compatibility list against the current R package documentation.
- Add examples for each existing method.
- Mark the features that users need most.
- Test the proposed shared visual rules with a small prototype.
