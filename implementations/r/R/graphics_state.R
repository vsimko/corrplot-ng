# Preserve caller-owned graphics state around legacy corrplot drawing.
#
# corrplot is expected to draw on the active device. Snapshotting writable
# parameters prevents its internal layout changes from leaking into the
# caller's subsequent base-graphics operations. The adapter never owns the
# device lifecycle.
.corrplot_ng_with_graphics_state <- function(expr) {
  original_device <- grDevices::dev.cur()
  original_parameters <- if (original_device > 1L) {
    graphics::par(no.readonly = TRUE)
  } else {
    NULL
  }

  on.exit({
    open_devices <- grDevices::dev.list()
    if (!is.null(open_devices) && original_device %in% open_devices) {
      if (grDevices::dev.cur() != original_device) {
        grDevices::dev.set(original_device)
      }
      if (!is.null(original_parameters)) {
        try(graphics::par(original_parameters), silent = TRUE)
      }
    }
  }, add = TRUE)

  force(expr)
}
