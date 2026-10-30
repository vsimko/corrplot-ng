#' Draw a standalone color legend
#' @param colbar Character vector of colors.
#' @param labels Numeric or character labels.
#' @param at Optional label positions normalized to [0, 1].
#' @param xlim,ylim Plot-coordinate limits.
#' @param vertical Whether to draw a vertical bar.
#' @param ratio.colbar Fraction of the legend dimension occupied by the bar.
#' @param lim.segment Two normalized tick-span coordinates, or NULL/"auto".
#' @param align Vertical label alignment: "c", "l", or "r".
#' @param addlabels Whether to draw labels.
#' @param ... Graphical arguments forwarded to graphics::text().
#' @return Invisibly returns NULL.
#' @export
colorlegend <- function(colbar, labels, at = NULL, xlim = c(0, 1),
  ylim = c(0, 1), vertical = TRUE, ratio.colbar = 0.4,
  lim.segment = "auto", align = c("c", "l", "r"), addlabels = TRUE, ...) {
  valid_limits <- function(x) is.numeric(x) && length(x) == 2L &&
    all(is.finite(x)) && diff(x) != 0
  if (!is.character(colbar) || !length(colbar) || anyNA(colbar))
    stop("colbar must be a non-empty character vector of colors", call. = FALSE)
  tryCatch(grDevices::col2rgb(colbar),
    error = function(e) stop("colbar contains an invalid color", call. = FALSE))
  if (missing(labels) || is.null(labels) || !length(labels) || anyNA(labels))
    stop("labels must be a non-empty vector without missing values", call. = FALSE)
  if (!valid_limits(xlim) || !valid_limits(ylim))
    stop("xlim and ylim must be finite numeric vectors of length 2 with distinct limits", call. = FALSE)
  if (!is.logical(vertical) || length(vertical) != 1L || is.na(vertical))
    stop("vertical must be TRUE or FALSE", call. = FALSE)
  if (!is.logical(addlabels) || length(addlabels) != 1L || is.na(addlabels))
    stop("addlabels must be TRUE or FALSE", call. = FALSE)
  if (!is.numeric(ratio.colbar) || length(ratio.colbar) != 1L ||
      !is.finite(ratio.colbar) || ratio.colbar <= 0 || ratio.colbar > 1)
    stop("ratio.colbar must be greater than 0 and at most 1", call. = FALSE)
  if (is.null(at) && addlabels) at <- seq(0, 1, length.out = length(labels))
  if (is.null(lim.segment) ||
      (is.character(lim.segment) && length(lim.segment) == 1L &&
       !is.na(lim.segment) && identical(lim.segment, "auto")))
    lim.segment <- ratio.colbar + c(0, ratio.colbar * 0.2)
  if (!is.null(at) && (!is.numeric(at) || anyNA(at) ||
      any(!is.finite(at)) || any(at < 0 | at > 1)))
    stop("at should be numeric positions between 0 and 1", call. = FALSE)
  if (addlabels && length(at) != length(labels))
    stop("at and labels must have the same length when addlabels is TRUE", call. = FALSE)
  if (!is.numeric(lim.segment) || length(lim.segment) != 2L ||
      anyNA(lim.segment) || any(!is.finite(lim.segment)) ||
      any(lim.segment < 0 | lim.segment > 1) || lim.segment[1] > lim.segment[2])
    stop("lim.segment must be an increasing numeric vector of length 2 between 0 and 1", call. = FALSE)

  align <- match.arg(align)
  xgap <- diff(xlim); ygap <- diff(ylim); len <- length(colbar)
  if (vertical) {
    at <- at * ygap + ylim[1]
    yy <- seq(ylim[1], ylim[2], length.out = len + 1L)
    graphics::rect(rep(xlim[1], len), yy[seq_len(len)],
      rep(xlim[1] + xgap * ratio.colbar, len), yy[-1L], col = colbar, border = colbar)
    graphics::rect(xlim[1], ylim[1], xlim[1] + xgap * ratio.colbar, ylim[2], border = "black")
    graphics::segments(xlim[1] + xgap * lim.segment[1], at,
      xlim[1] + xgap * lim.segment[2], at)
    if (addlabels) {
      xp <- rep(xlim[1] + xgap * max(lim.segment, ratio.colbar), length(at))
      if (align == "l") {
        graphics::text(xp, y = at, labels = labels, pos = 4, ...)
      } else if (align == "r") {
        graphics::text(xlim[2], y = at, labels = labels, pos = 2, ...)
      } else {
        graphics::text((xp + xlim[2]) / 2, y = at, labels = labels, ...)
      }
    }
  } else {
    at <- at * xgap + xlim[1]
    xx <- seq(xlim[1], xlim[2], length.out = len + 1L)
    graphics::rect(xx[seq_len(len)], rep(ylim[2] - ratio.colbar * ygap, len),
      xx[-1L], rep(ylim[2], len), col = colbar, border = colbar)
    graphics::rect(xlim[1], ylim[2] - ratio.colbar * ygap, xlim[2], ylim[2], border = "black")
    graphics::segments(at, ylim[2] - ygap * lim.segment[1],
      at, ylim[2] - ygap * lim.segment[2])
    if (addlabels) {
      yp <- rep(ylim[2] - ygap * max(lim.segment, ratio.colbar), length(at))
      graphics::text(x = at, y = yp, labels = labels, pos = 1, ...)
    }
  }
  invisible(NULL)
}
