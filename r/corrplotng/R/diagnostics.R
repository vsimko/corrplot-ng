#' Read a structured corrplot-ng diagnostic
#'
#' @param condition A condition caught from a corrplot-ng operation.
#' @return A language-neutral diagnostic-v1 list, or NULL when the condition
#'   is not a corrplot-ng diagnostic.
#' @export
corrplot_ng_diagnostic <- function(condition) {
  if (!inherits(condition, "corrplot_ng_diagnostic")) {
    return(NULL)
  }
  condition$diagnostic
}

.corrplot_ng_abort <- function(code, message, stage, path = NULL,
                               row = NULL, column = NULL, details = NULL) {
  diagnostic <- list(
    schema_version = 1L,
    code = code,
    severity = "error",
    stage = stage,
    message = message
  )
  if (!is.null(path)) diagnostic$path <- path
  if (!is.null(row)) diagnostic$row <- as.integer(row)
  if (!is.null(column)) diagnostic$column <- as.integer(column)
  if (!is.null(details)) diagnostic$details <- details
  condition <- structure(
    list(message = message, call = NULL, diagnostic = diagnostic),
    class = c("corrplot_ng_diagnostic", "corrplot_ng_error", "error", "condition")
  )
  stop(condition)
}

.corrplot_ng_warn <- function(code, message, stage, path = NULL,
                              row = NULL, column = NULL, details = NULL) {
  diagnostic <- list(
    schema_version = 1L,
    code = code,
    severity = "warning",
    stage = stage,
    message = message
  )
  if (!is.null(path)) diagnostic$path <- path
  if (!is.null(row)) diagnostic$row <- as.integer(row)
  if (!is.null(column)) diagnostic$column <- as.integer(column)
  if (!is.null(details)) diagnostic$details <- details
  condition <- structure(
    list(message = message, call = NULL, diagnostic = diagnostic),
    class = c("corrplot_ng_diagnostic", "corrplot_ng_warning", "warning", "condition")
  )
  warning(condition)
}
