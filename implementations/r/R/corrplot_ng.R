#' Parse a legacy corrplot label
#'
#' Labels prefixed with `: `, `=`, or `$` (without the space after the
#' prefix) are parsed as native R plotmath expressions. Other labels are
#' returned unchanged as character strings.
#'
#' @param label One non-missing character string.
#' @return A character string or a native R expression.
#' @export
corrplot_ng_parse_label <- function(label) {
  if (!is.character(label) || length(label) != 1L || is.na(label)) {
    stop("label must be one non-missing character string", call. = FALSE)
  }
  if (startsWith(label, ":") || startsWith(label, "=") || startsWith(label, "$")) {
    source <- substring(label, 2L)
    tryCatch(parse(text = source), error = function(e) {
      stop("invalid plotmath label '", label, "': ", conditionMessage(e), call. = FALSE)
    })
  } else {
    label
  }
}

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
    supported <- c("annotation", "method", "type", "diag", "addCoef.col",
      "addCoefasPercent", "number.digits", "number.cex", "number.font", "tl.pos", "tl.offset", "tl.srt")
    unsupported <- setdiff(arg_names, supported)
    if ("method" %in% arg_names) {
      requested_method <- args$method
      if (length(requested_method) > 1L && !identical(requested_method, c("circle", "number"))) {
        unsupported <- unique(c(unsupported, "method"))
      } else if (length(requested_method) == 1L && !requested_method %in% c("circle", "number")) {
        unsupported <- unique(c(unsupported, "method"))
      } else if (length(requested_method) == 0L) {
        unsupported <- unique(c(unsupported, "method"))
      }
    }
    if (length(unsupported) > 0L) stop("The scene engine does not support: ", paste(unsupported, collapse = ", "), ". Use engine = 'legacy' for these options.", call. = FALSE)
  }
  defaults <- list(annotation = "", method = "circle", type = "full", diag = TRUE,
    addCoef.col = NULL, addCoefasPercent = FALSE, number.digits = NULL,
    number.cex = 1, number.font = 2,\n    tl.pos = NULL, tl.offset = 0.4, tl.srt = 90)
  options <- utils::modifyList(defaults, args)
  .corrplot_ng_build_scene_v1(corr, options)
}

.corrplot_ng_build_scene_v1 <- function(corr, options = list()) {
  if (!is.matrix(corr) || !is.numeric(corr)) stop("corr must be a numeric matrix for engine = 'scene'", call. = FALSE)
  n <- nrow(corr)
  if (n < 1L || ncol(corr) != n) stop("the scene engine requires a non-empty square matrix", call. = FALSE)
  observed <- !is.na(corr)
  if (any(!is.finite(corr[observed])) || any(corr[observed] < -1 | corr[observed] > 1)) stop("the scene engine requires finite observed values in [-1, 1]", call. = FALSE)

  annotation <- options$annotation
  if (!is.character(annotation) || length(annotation) != 1L || is.na(annotation)) stop("annotation must be one non-missing character string", call. = FALSE)
  method <- match.arg(options$method, c("circle", "number"))
  display_type <- match.arg(options$type, c("full", "upper", "lower"))
  if (!is.logical(options$diag) || length(options$diag) != 1L || is.na(options$diag)) stop("diag must be one non-missing logical value", call. = FALSE)
  if (!is.logical(options$addCoefasPercent) || length(options$addCoefasPercent) != 1L || is.na(options$addCoefasPercent)) stop("addCoefasPercent must be one non-missing logical value", call. = FALSE)
  if (!is.null(options$addCoef.col) && (!is.character(options$addCoef.col) || length(options$addCoef.col) != 1L || is.na(options$addCoef.col) || !nzchar(options$addCoef.col))) stop("addCoef.col must be one non-empty color string or NULL", call. = FALSE)
  if (!is.null(options$addCoef.col)) tryCatch(grDevices::col2rgb(options$addCoef.col), error = function(e) stop("addCoef.col must be a valid R color", call. = FALSE))
  digits <- options$number.digits
  if (is.null(digits)) digits <- if (options$addCoefasPercent) 0L else 2L
  if (!is.numeric(digits) || length(digits) != 1L || is.na(digits) || !is.finite(digits) || digits < 0 || digits %% 1 != 0) stop("number.digits must be a non-negative integer", call. = FALSE)
  cex <- options$number.cex
  if (!is.numeric(cex) || length(cex) != 1L || is.na(cex) || !is.finite(cex) || cex <= 0) stop("number.cex must be greater than 0", call. = FALSE)
  font <- options$number.font
  if (!is.numeric(font) || length(font) != 1L || is.na(font) || !(font %in% 1:4)) stop("number.font must be one of 1, 2, 3, or 4", call. = FALSE)

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
  text_annotations <- list()
  index <- 1L
  annotation_index <- 1L
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
      visible <- !is.na(value) && (options$diag || row != column) &&
        (display_type == "full" || (display_type == "upper" && row <= column) ||
          (display_type == "lower" && row >= column))
      role <- if (method == "number") "value" else if (!is.null(options$addCoef.col)) "coefficient" else NULL
      if (visible && !is.null(role)) {
        label_value <- value * if (options$addCoefasPercent) 100 else 1
        label_color <- if (role == "coefficient") options$addCoef.col else
          if (value > 0) "#2166ac" else if (value < 0) "#b2182b" else "#bdbdbd"
        text_annotations[[annotation_index]] <- list(
          row = row - 1L, column = column - 1L, role = role,
          text = format(round(label_value, digits), nsmall = digits,
            trim = TRUE, scientific = FALSE),
          color = label_color, font_size = 13 * cex,
          font_weight = if (font %in% c(2, 4)) "bold" else "normal",
          font_style = if (font %in% c(3, 4)) "italic" else "normal")
        annotation_index <- annotation_index + 1L
      }
      index <- index + 1L
    }
  }
  tl_pos <- options$tl.pos
  if (identical(tl_pos, FALSE)) tl_pos <- "n"
  if (is.null(tl_pos) || identical(tl_pos, TRUE)) {
    tl_pos <- switch(display_type, full = "lt", lower = "ld", upper = "td")
  }
  if (!is.character(tl_pos) || length(tl_pos) != 1L || is.na(tl_pos) ||
      !tl_pos %in% c("lt", "ld", "td", "d", "l", "n")) {
    stop("tl.pos must be one of 'lt', 'ld', 'td', 'd', 'l', or 'n'", call. = FALSE)
  }
  if (tl_pos == "ld" && display_type != "lower") {
    stop("tl.pos = 'ld' requires type = 'lower'", call. = FALSE)
  }
  if (tl_pos == "td" && display_type != "upper") {
    stop("tl.pos = 'td' requires type = 'upper'", call. = FALSE)
  }
  if (!is.numeric(options$tl.offset) || length(options$tl.offset) != 1L ||
      is.na(options$tl.offset) || !is.finite(options$tl.offset)) {
    stop("tl.offset must be one finite number", call. = FALSE)
  }
  if (!is.numeric(options$tl.srt) || length(options$tl.srt) != 1L ||
      is.na(options$tl.srt) || !is.finite(options$tl.srt)) {
    stop("tl.srt must be one finite number", call. = FALSE)
  }
  show_rows <- tl_pos %in% c("lt", "ld", "td", "l")
  show_columns <- tl_pos %in% c("lt", "ld", "td", "d")
  label_offset <- 13 * 0.6 * options$tl.offset
  row_position <- if (tl_pos == "td") "diagonal" else "left"
  column_position <- if (tl_pos %in% c("ld", "d")) "diagonal" else "top"
  label_items <- lapply(seq_len(n), function(i) {
    raw <- labels[[i]]
    prefix <- if (startsWith(raw, ":") || startsWith(raw, "=") || startsWith(raw, "$")) substr(raw, 1L, 1L) else NULL
    source <- if (is.null(prefix)) NULL else substring(raw, 2L)
    corrplot_ng_parse_label(raw)
    list(text = raw, row_x = if (row_position == "left") left_margin - 12 - label_offset else
        left_margin + (i - 1L) * cell_size,
      row_y = top_margin + (i - 1L) * cell_size + cell_size / 2,
      column_x = left_margin + (i - 1L) * cell_size + cell_size / 2,
      column_y = if (column_position == "top") top_margin - 16 - label_offset else
        top_margin + (i - 1L) * cell_size + cell_size / 2,
      row_visible = show_rows, column_visible = show_columns,
      row_position = row_position, column_position = column_position,
      column_rotation = options$tl.srt, offset = options$tl.offset,
      plotmath = if (is.null(source)) NULL else list(prefix = prefix, source = source))
  })
  scene <- list(schema_version = 1L, kind = "correlation_plot",
    width = left_margin + n * cell_size + 24, height = top_margin + n * cell_size + 64,
    labels = label_items, cells = cells, annotation = annotation,
    glyph_method = method, text_annotations = text_annotations,
    title = "", description = "",
    style = list(positive_color = "#2166ac", negative_color = "#b2182b", zero_color = "#bdbdbd", missing_fill = "#f2f2f2", grid_color = "#e0e0e0"))
  class(scene) <- c("corrplot_ng_scene", "list")
  scene
}
