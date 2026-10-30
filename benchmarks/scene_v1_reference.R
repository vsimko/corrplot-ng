# Pure R implementation of the experimental scene-v1 contract.
# Kept separate from the legacy corrplot adapter and native prototypes.

build_scene_v1_r <- function(labels, matrix, annotation = "") {
  n <- length(labels)
  if (n < 1L || !is.matrix(matrix) || nrow(matrix) != n || ncol(matrix) != n) {
    stop("labels and matrix dimensions must agree and be non-empty")
  }
  if (anyNA(labels) || !is.character(labels)) stop("labels must be non-missing character values")
  if (!is.character(annotation) || length(annotation) != 1L || is.na(annotation)) {
    stop("annotation must be one non-missing string")
  }
  if (any(!is.na(matrix) & (!is.finite(matrix) | matrix < -1 | matrix > 1))) {
    stop("matrix values must be finite, NA, or between -1 and 1")
  }

  cell <- 84
  left <- 104
  top <- 76
  cells <- lapply(seq_len(n * n), function(k) {
    row <- (k - 1L) %/% n
    column <- (k - 1L) %% n
    value <- matrix[row + 1L, column + 1L]
    list(
      row = row, column = column,
      value = if (is.na(value)) NULL else value,
      x = left + column * cell, y = top + row * cell,
      center_x = left + column * cell + cell / 2,
      center_y = top + row * cell + cell / 2, size = cell
    )
  })
  label_items <- lapply(seq_len(n), function(i) {
    list(
      text = labels[[i]], row_x = 92,
      row_y = top + (i - 1L) * cell + cell / 2,
      column_x = left + (i - 1L) * cell + cell / 2, column_y = 60
    )
  })
  list(
    schema_version = 1L, kind = "correlation_plot",
    width = left + n * cell + 24, height = top + n * cell + 64,
    labels = label_items, cells = cells, annotation = annotation,
    style = list(
      positive_color = "#2166ac", negative_color = "#b2182b",
      zero_color = "#bdbdbd", missing_fill = "#f2f2f2",
      grid_color = "#e0e0e0"
    )
  )
}

encode_scene_v1_r <- function(scene) {
  jsonlite::toJSON(scene, auto_unbox = TRUE, null = "null", na = "null",
                   digits = NA, pretty = FALSE)
}

build_scene_json_r <- function(labels, matrix, annotation = "") {
  encode_scene_v1_r(build_scene_v1_r(labels, matrix, annotation))
}
