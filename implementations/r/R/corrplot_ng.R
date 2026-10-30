corrplot_ng <- function(corr, ..., engine = c("legacy", "scene")) {
  engine <- match.arg(engine)
  args <- list(...)
  if (identical(engine, "legacy")) {
    if ("annotation" %in% names(args)) stop("annotation is a scene-v1 option; use engine = 'scene'", call. = FALSE)
    return(do.call(corrplot::corrplot, c(list(corr = corr), args)))
  }
  if (length(args) > 0L) {
    arg_names <- names(args)
    if (is.null(arg_names) || anyNA(arg_names) || any(!nzchar(arg_names)) || anyDuplicated(arg_names)) stop("The scene engine requires uniquely named arguments", call. = FALSE)
    unsupported <- setdiff(arg_names, "annotation")
    if (length(unsupported) > 0L) stop("The scene engine does not support: ", paste(unsupported, collapse = ", "), ". Use engine = 'legacy' for these options.", call. = FALSE)
  }
  annotation <- if ("annotation" %in% names(args)) args[["annotation"]] else ""
  .corrplot_ng_build_scene_v1(corr, annotation = annotation)
}
.corrplot_ng_build_scene_v1 <- function(corr, annotation = "") {
  if (!is.matrix(corr) || !is.numeric(corr)) stop("corr must be a numeric matrix for engine = 'scene'", call. = FALSE)
  n <- nrow(corr)
  if (n < 1L || ncol(corr) != n) stop("the scene engine requires a non-empty square matrix", call. = FALSE)
  observed <- !is.na(corr)
  if (any(!is.finite(corr[observed])) || any(corr[observed] < -1 | corr[observed] > 1)) stop("the scene engine requires finite observed values in [-1, 1]", call. = FALSE)
  if (!is.character(annotation) || length(annotation) != 1L || is.na(annotation)) stop("annotation must be one non-missing character string", call. = FALSE)
  row_labels <- rownames(corr)
  column_labels <- colnames(corr)
  if (is.null(row_labels) && is.null(column_labels)) {
    labels <- as.character(seq_len(n))
  } else if (is.null(row_labels)) {
    labels <- column_labels
  } else if (is.null(column_labels)) {
    labels <- row_labels
  } else {
    if (!identical(row_labels, column_labels)) stop("row and column names must match for the scene engine", call. = FALSE)
    labels <- row_labels
  }
  if (!is.character(labels) || length(labels) != n || anyNA(labels)) stop("matrix labels must be non-missing character values", call. = FALSE)
  cell_size <- 84
  left_margin <- 104
  top_margin <- 76
  cells <- vector("list", n * n)
  index <- 1L
  for (row in seq_len(n)) {
    for (column in seq_len(n)) {
      value <- corr[row, column]
      cells[[index]] <- list(row = row - 1L, column = column - 1L,
        value = if (is.na(value)) NULL else as.numeric(value),
        x = left_margin + (column - 1L) * cell_size,
        y = top_margin + (row - 1L) * cell_size,
        center_x = left_margin + (column - 1L) * cell_size + cell_size / 2,
        center_y = top_margin + (row - 1L) * cell_size + cell_size / 2,
        size = cell_size)
      index <- index + 1L
    }
  }
  label_items <- lapply(seq_len(n), function(i) list(text = labels[[i]], row_x = 92,
    row_y = top_margin + (i - 1L) * cell_size + cell_size / 2,
    column_x = left_margin + (i - 1L) * cell_size + cell_size / 2, column_y = 60))
  scene <- list(schema_version = 1L, kind = "correlation_plot",
    width = left_margin + n * cell_size + 24, height = top_margin + n * cell_size + 64,
    labels = label_items, cells = cells, annotation = annotation,
    style = list(positive_color = "#2166ac", negative_color = "#b2182b", zero_color = "#bdbdbd", missing_fill = "#f2f2f2", grid_color = "#e0e0e0"))
  class(scene) <- c("corrplot_ng_scene", "list")
  scene
}
