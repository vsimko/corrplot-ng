# Preserve caller-owned graphics state around legacy corrplot drawing.
#
# Successful plots establish a coordinate system that subsequent add=TRUE
# calls must reuse. Restore presentation parameters after success, but retain
# plot geometry. On errors, restore the complete writable parameter snapshot.
# The adapter never opens or closes the caller's graphics device.
.corrplot_ng_with_graphics_state <- function(expr) {
  original_device <- grDevices::dev.cur()
  original_parameters <- if (original_device > 1L) {
    graphics::par(no.readonly = TRUE)
  } else {
    NULL
  }
  completed <- FALSE
  geometry_parameters <- c(
    "fig", "fin", "mai", "mar", "mfcol", "mfg", "mfrow", "new",
    "oma", "omd", "omi", "page", "pin", "plt", "pty", "usr",
    "xaxp", "xaxs", "xaxt", "xlog", "yaxp", "yaxs", "yaxt", "ylog"
  )

  on.exit({
    open_devices <- grDevices::dev.list()
    if (!is.null(open_devices) && original_device %in% open_devices) {
      if (grDevices::dev.cur() != original_device) {
        grDevices::dev.set(original_device)
      }
      if (!is.null(original_parameters)) {
        restore <- original_parameters
        if (completed) {
          restore <- restore[setdiff(names(restore), geometry_parameters)]
        }
        if (length(restore)) {
          try(graphics::par(restore), silent = TRUE)
        }
      }
    }
  }, add = TRUE)

  result <- force(expr)
  completed <- TRUE
  result
}
