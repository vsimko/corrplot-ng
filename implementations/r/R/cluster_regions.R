# Renderer-neutral cluster-region input helpers and legacy R drawing adapters.
#
# The region request shape matches the input accepted by the shared scene-v1
# cluster-region builder: id plus exactly one of variables/indices, color, and
# line_width. Geometry is resolved by the scene builder, not by this adapter.

.corrplot_ng_region_style <- function(col, lwd) {
  if (length(col) != 1L || is.na(col)) {
    stop("col must be one valid R color", call. = FALSE)
  }
  rgb <- tryCatch(grDevices::col2rgb(col), error = function(error) {
    stop("col must be a valid R color", call. = FALSE)
  })
  if (!is.numeric(lwd) || length(lwd) != 1L || is.na(lwd) ||
      !is.finite(lwd) || lwd < 0.1 || lwd > 12) {
    stop("lwd must be one finite number between 0.1 and 12", call. = FALSE)
  }
  list(
    color = sprintf("#%02X%02X%02X", rgb[1L, 1L], rgb[2L, 1L], rgb[3L, 1L]),
    line_width = as.numeric(lwd)
  )
}

.corrplot_ng_region_inputs <- function(
    starts, ends, labels, source_indices, col, lwd) {
  if (length(starts) != length(ends) || !length(starts)) {
    stop("at least one non-empty cluster interval is required", call. = FALSE)
  }
  style <- .corrplot_ng_region_style(col, lwd)
  has_unique_labels <- !is.null(labels) && length(labels) &&
    !anyNA(labels) && !anyDuplicated(labels)
  lapply(seq_along(starts), function(i) {
    members <- seq.int(starts[[i]], ends[[i]])
    region <- list(id = sprintf("corrplot-cluster-%03d", i))
    if (has_unique_labels) {
      region$variables <- as.list(labels[members])
    } else {
      region$indices <- as.list(as.integer(source_indices[members] - 1L))
    }
    region$color <- style$color
    region$line_width <- style$line_width
    region
  })
}

.corrplot_ng_region_request_intervals <- function(requests, labels, source_indices) {
  intervals <- lapply(requests, function(region) {
    if (!is.null(region$variables)) {
      positions <- match(unlist(region$variables, use.names = FALSE), labels)
    } else {
      requested_indices <- as.integer(unlist(region$indices, use.names = FALSE)) + 1L
      positions <- match(requested_indices, source_indices)
    }
    if (!length(positions) || anyNA(positions)) {
      stop("cluster-region request could not be mapped back to the displayed matrix", call. = FALSE)
    }
    c(start = min(positions), end = max(positions))
  })
  list(
    starts = vapply(intervals, function(x) x[["start"]], integer(1)),
    ends = vapply(intervals, function(x) x[["end"]], integer(1))
  )
}

.corrplot_ng_validate_intervals <- function(index, n) {
  if (!is.numeric(index) || is.logical(index) || length(index) < 2L ||
      anyNA(index) || any(!is.finite(index)) ||
      any(index != as.integer(index)) || any(index < 1L | index > n) ||
      any(diff(index) <= 0)) {
    stop("index must be a strictly increasing vector of at least two integer positions within the matrix", call. = FALSE)
  }
  index <- as.integer(index)
  list(starts = index[-length(index)], ends = index[-1L])
}

.corrplot_ng_validate_corr_result <- function(corrRes, need_positions = FALSE) {
  if (!is.list(corrRes) || !is.matrix(corrRes$corr) ||
      !is.numeric(corrRes$corr) || nrow(corrRes$corr) < 1L ||
      nrow(corrRes$corr) != ncol(corrRes$corr)) {
    stop("corrRes must be a corrplot result containing a non-empty square numeric corr matrix", call. = FALSE)
  }
  corr <- corrRes$corr
  row_names <- rownames(corr)
  column_names <- colnames(corr)
  axis_names_compatible <- (is.null(row_names) && is.null(column_names)) ||
    (!is.null(row_names) && !is.null(column_names) &&
      identical(row_names, column_names))
  labels <- if (axis_names_compatible && !is.null(row_names)) row_names else NULL
  type <- corrRes$arg$type
  if (is.null(type)) type <- "full"
  if (!is.character(type) || length(type) != 1L || is.na(type) ||
      !type %in% c("full", "upper", "lower")) {
    stop("corrRes$arg$type must be 'full', 'upper', or 'lower'", call. = FALSE)
  }
  positions <- corrRes$corrPos
  if (need_positions &&
      (!is.data.frame(positions) ||
       !all(c("xName", "yName", "x", "y") %in% names(positions)))) {
    stop("corrRes$corrPos must contain xName, yName, x, and y columns for namesMat", call. = FALSE)
  }
  list(
    corr = corr, labels = labels, type = type, positions = positions,
    axis_names_compatible = axis_names_compatible
  )
}

.corrplot_ng_draw_index_regions <- function(starts, ends, n, type, col, lwd, ...) {
  x1 <- starts - 0.5
  x2 <- ends + 0.5
  y1 <- n - starts + 0.5
  y2 <- n - ends + 0.5
  segments <- rbind(
    cbind(x1, y1, x2, y1),
    cbind(x1, y1, x1, y2),
    cbind(x2, y2, x2, y1),
    cbind(x2, y2, x1, y2)
  )
  if (type != "full") {
    mirrored <- segments
    mirrored[, 2L] <- abs(mirrored[, 2L] - n - 1)
    mirrored[, 4L] <- abs(mirrored[, 4L] - n - 1)
    if (type == "upper") {
      keep <- (mirrored[, 1L] - mirrored[, 2L] > -0.1) &
        (mirrored[, 3L] - mirrored[, 4L] > -0.1)
    } else {
      keep <- (mirrored[, 2L] - mirrored[, 1L] > -0.1) &
        (mirrored[, 4L] - mirrored[, 3L] > -0.1)
    }
    segments <- segments[keep, , drop = FALSE]
  }
  if (nrow(segments)) {
    graphics::segments(
      segments[, 1L], segments[, 2L], segments[, 3L], segments[, 4L],
      col = col, lwd = lwd, ...
    )
  }
  invisible(segments)
}

.corrplot_ng_names_matrix <- function(namesMat) {
  if (is.character(namesMat) && is.null(dim(namesMat))) {
    if (length(namesMat) != 4L) {
      stop("namesMat must be a four-element character vector or a matrix with four columns", call. = FALSE)
    }
    namesMat <- matrix(namesMat, nrow = 1L)
  }
  if (!is.matrix(namesMat) || !is.character(namesMat) ||
      ncol(namesMat) != 4L || nrow(namesMat) < 1L ||
      anyNA(namesMat) || any(!nzchar(namesMat))) {
    stop("namesMat must be a non-empty character matrix with four columns and no missing names", call. = FALSE)
  }
  namesMat
}

.corrplot_ng_lookup_named_cell <- function(pair, positions, row_number) {
  matches <- which(positions$xName == pair[[1L]] & positions$yName == pair[[2L]])
  if (!length(matches)) {
    stop(
      "namesMat row ", row_number, ": paired X-Y names '",
      pair[[1L]], "', '", pair[[2L]], "' were not found",
      call. = FALSE
    )
  }
  if (length(matches) > 1L) {
    stop(
      "namesMat row ", row_number, ": paired X-Y names are ambiguous in corrPos",
      call. = FALSE
    )
  }
  c(x = positions$x[[matches]], y = positions$y[[matches]])
}

.corrplot_ng_namesmat_region_inputs <- function(
    namesMat, corr, positions, col, lwd) {
  matrix_names <- rownames(corr)
  if (is.null(matrix_names) || is.null(colnames(corr)) ||
      !identical(matrix_names, colnames(corr)) ||
      anyNA(matrix_names) || anyDuplicated(matrix_names)) {
    return(list())
  }
  namesMat <- .corrplot_ng_names_matrix(namesMat)
  n <- nrow(corr)
  requests <- list()
  for (i in seq_len(nrow(namesMat))) {
    first <- .corrplot_ng_lookup_named_cell(namesMat[i, 1:2], positions, i)
    second <- .corrplot_ng_lookup_named_cell(namesMat[i, 3:4], positions, i)
    columns <- sort(as.integer(c(first[["x"]], second[["x"]])))
    rows <- sort(as.integer(n - c(first[["y"]], second[["y"]]) + 1L))
    if (identical(columns, rows)) {
      requests <- c(requests, .corrplot_ng_region_inputs(
        starts = columns[[1L]], ends = columns[[2L]],
        labels = matrix_names, source_indices = seq_len(n),
        col = col, lwd = lwd
      ))
    }
  }
  requests
}

#' Add cluster rectangles to a legacy corrplot
#'
#' Draws index-, name-, or cell-name-defined rectangles on the active device.
#' Index and name ranges also resolve to the shared cluster-region request
#' shape used by the scene-v1 builder.
#'
#' @param corrRes Invisible result returned by corrplot_ng() or corrplot_ng_mixed().
#' @param index Increasing one-based boundaries; each adjacent pair defines
#'   one inclusive cluster range.
#' @param name Ordered variable names defining the same boundary ranges as
#'   index.
#' @param namesMat A four-element character vector or a matrix with columns
#'   xleft, ybottom, xright, and ytop, resolved through corrRes$corrPos.
#' @param col R graphics color used for the border.
#' @param lwd Border line width, between 0.1 and 12.
#' @param ... Additional arguments passed to graphics::segments() or rect().
#' @return Invisibly returns corrRes unchanged.
#' @export
corrRect <- function(corrRes = NULL, index = NULL, name = NULL,
                     namesMat = NULL, col = "black", lwd = 2, ...) {
  supplied <- c(!is.null(index), !is.null(name), !is.null(namesMat))
  if (sum(supplied) != 1L) {
    stop("supply exactly one of index, name, or namesMat", call. = FALSE)
  }
  validated <- .corrplot_ng_validate_corr_result(
    corrRes, need_positions = !is.null(namesMat)
  )
  corr <- validated$corr
  n <- nrow(corr)
  if (!is.null(name)) {
    if (is.null(validated$labels) || anyDuplicated(validated$labels)) {
      stop("row and column names must be identical and unique when name is used", call. = FALSE)
    }
    if (!is.character(name) || length(name) < 2L || anyNA(name) ||
        any(!nzchar(name)) || anyDuplicated(name) ||
        !all(name %in% validated$labels)) {
      stop("name must contain at least two unique existing variable names", call. = FALSE)
    }
    index <- match(name, validated$labels)
  }
  if (!is.null(index)) {
    if (!validated$axis_names_compatible) {
      stop("row and column names must be identical when index is used", call. = FALSE)
    }
    intervals <- .corrplot_ng_validate_intervals(index, n)
    requests <- .corrplot_ng_region_inputs(
      intervals$starts, intervals$ends, validated$labels,
      seq_len(n), col, lwd
    )
    resolved <- .corrplot_ng_region_request_intervals(
      requests, validated$labels, seq_len(n)
    )
    .corrplot_ng_draw_index_regions(
      resolved$starts, resolved$ends, n, validated$type, col, lwd, ...
    )
  } else {
    names_matrix <- .corrplot_ng_names_matrix(namesMat)
    .corrplot_ng_namesmat_region_inputs(
      names_matrix, corr, validated$positions, col, lwd
    )
    first <- lapply(seq_len(nrow(names_matrix)), function(i) {
      .corrplot_ng_lookup_named_cell(names_matrix[i, 1:2], validated$positions, i)
    })
    second <- lapply(seq_len(nrow(names_matrix)), function(i) {
      .corrplot_ng_lookup_named_cell(names_matrix[i, 3:4], validated$positions, i)
    })
    graphics::rect(
      vapply(seq_along(first), function(i) min(first[[i]][["x"]], second[[i]][["x"]]) - 0.5, numeric(1)),
      vapply(seq_along(first), function(i) min(first[[i]][["y"]], second[[i]][["y"]]) - 0.5, numeric(1)),
      vapply(seq_along(first), function(i) max(first[[i]][["x"]], second[[i]][["x"]]) + 0.5, numeric(1)),
      vapply(seq_along(first), function(i) max(first[[i]][["y"]], second[[i]][["y"]]) + 0.5, numeric(1)),
      border = col, lwd = lwd, ...
    )
  }
  invisible(corrRes)
}

# Resolve hclust cuts into contiguous leaf-order region requests.
.corrplot_ng_hclust_region_data <- function(
    corr, k, col, lwd, method) {
  if (!is.matrix(corr) || !is.numeric(corr) || nrow(corr) < 2L ||
      nrow(corr) != ncol(corr) || anyNA(corr) || any(!is.finite(corr))) {
    stop("corr must be a finite, non-empty square numeric matrix", call. = FALSE)
  }
  n <- nrow(corr)
  if (!is.numeric(k) || length(k) != 1L || is.na(k) ||
      !is.finite(k) || k != as.integer(k) || k < 1L || k > n) {
    stop("k must be one integer between 1 and nrow(corr)", call. = FALSE)
  }
  method <- match.arg(method)
  style <- .corrplot_ng_region_style(col, lwd)
  tree <- stats::hclust(stats::as.dist(1 - corr), method = method)
  clusters <- stats::cutree(tree, k = as.integer(k))
  ordered_clusters <- clusters[tree$order]
  runs <- rle(as.integer(ordered_clusters))
  ends <- cumsum(runs$lengths)
  starts <- ends - runs$lengths + 1L
  ordered_labels <- rownames(corr)
  if (is.null(ordered_labels)) ordered_labels <- as.character(seq_len(n))
  ordered_labels <- ordered_labels[tree$order]
  if (anyNA(ordered_labels) || anyDuplicated(ordered_labels)) {
    ordered_labels <- NULL
  }
  requests <- .corrplot_ng_region_inputs(
    starts, ends, ordered_labels, tree$order, col, lwd
  )
  resolved <- .corrplot_ng_region_request_intervals(
    requests, ordered_labels, tree$order
  )
  list(
    starts = resolved$starts,
    ends = resolved$ends,
    order = as.integer(tree$order),
    labels = ordered_labels,
    style = style,
    requests = requests
  )
}

#' Add rectangles for hierarchical clusters
#'
#' Clusters the input matrix using hclust(as.dist(1 - corr)) and draws one
#' rectangle for each cut cluster in dendrogram leaf order.
#'
#' @param corr Numeric square correlation matrix.
#' @param k Number of clusters to draw.
#' @param col R graphics color used for the border.
#' @param lwd Border line width, between 0.1 and 12.
#' @param method Hierarchical clustering method supported by corrplot 0.95.
#' @return Invisibly returns NULL, matching the legacy drawing helper.
#' @export
corrRect.hclust <- function(
    corr, k = 2, col = "black", lwd = 2,
    method = c("complete", "ward", "ward.D", "ward.D2", "single",
               "average", "mcquitty", "median", "centroid")) {
  regions <- .corrplot_ng_hclust_region_data(corr, k, col, lwd, method)
  n <- nrow(corr)
  for (i in seq_along(regions$requests)) {
    graphics::rect(
      regions$starts[[i]] - 0.5,
      n - regions$ends[[i]] + 0.5,
      regions$ends[[i]] + 0.5,
      n - regions$starts[[i]] + 0.5,
      border = col,
      lwd = regions$style$line_width
    )
  }
  invisible(NULL)
}
