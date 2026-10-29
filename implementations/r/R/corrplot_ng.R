#' Provisional corrplot-ng interface
#'
#' This is a temporary compatibility adapter. It delegates all work to the
#' original `corrplot` package and does not implement a new plotting engine.
#' Its purpose is to establish an R-facing API and test harness that can later
#' be redirected to a new implementation.
#'
#' @param corr A correlation matrix accepted by [corrplot::corrplot()].
#' @param ... Arguments passed unchanged to [corrplot::corrplot()].
#'
#' @return The result returned by [corrplot::corrplot()].
#' @export
corrplot_ng <- function(corr, ...) {
  corrplot::corrplot(corr = corr, ...)
}
