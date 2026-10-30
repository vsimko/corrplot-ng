#' Get sequential colors
#'
#' Return a legacy corrplot 0.95 sequential palette.
#'
#' @param sequential Sequential palette name. One of Oranges, Purples, Reds,
#'   Blues, Greens, Greys, OrRd, YlOrRd, YlOrBr, or YlGn.
#' @param n Number of colors to return; must be a positive integer.
#' @return A character vector of hexadecimal colors.
#' @seealso [grDevices::colorRampPalette()]
#' @export
COL1 <- function(sequential = c("Oranges", "Purples", "Reds", "Blues", "Greens",
                                "Greys", "OrRd", "YlOrRd", "YlOrBr", "YlGn"),
                 n = 200) {
  sequential <- match.arg(sequential)
  .corrplot_ng_validate_palette_length(n)
  colors <- switch(
    sequential,
    Oranges = c("#FFF5EB", "#FEE6CE", "#FDD0A2", "#FDAE6B", "#FD8D3C",
                "#F16913", "#D94801", "#A63603", "#7F2704"),
    Purples = c("#FCFBFD", "#EFEDF5", "#DADAEB", "#BCBDDC", "#9E9AC8",
                "#807DBA", "#6A51A3", "#54278F", "#3F007D"),
    Reds = c("#FFF5F0", "#FEE0D2", "#FCBBA1", "#FC9272", "#FB6A4A",
             "#EF3B2C", "#CB181D", "#A50F15", "#67000D"),
    Blues = c("#F7FBFF", "#DEEBF7", "#C6DBEF", "#9ECAE1", "#6BAED6",
              "#4292C6", "#2171B5", "#08519C", "#08306B"),
    Greens = c("#F7FCF5", "#E5F5E0", "#C7E9C0", "#A1D99B", "#74C476",
               "#41AB5D", "#238B45", "#006D2C", "#00441B"),
    Greys = c("#FFFFFF", "#F0F0F0", "#D9D9D9", "#BDBDBD", "#969696",
              "#737373", "#525252", "#252525", "#000000"),
    OrRd = c("#FFF7EC", "#FEE8C8", "#FDD49E", "#FDBB84", "#FC8D59",
             "#EF6548", "#D7301F", "#B30000", "#7F0000"),
    YlOrRd = c("#FFFFCC", "#FFEDA0", "#FED976", "#FEB24C", "#FD8D3C",
               "#FC4E2A", "#E31A1C", "#BD0026", "#800026"),
    YlOrBr = c("#FFFFE5", "#FFF7BC", "#FEE391", "#FEC44F", "#FE9929",
               "#EC7014", "#CC4C02", "#993404", "#662506"),
    YlGn = c("#FFFFE5", "#F7FCB9", "#D9F0A3", "#ADDD8E", "#78C679",
             "#41AB5D", "#238443", "#006837", "#004529")
  )
  grDevices::colorRampPalette(colors)(n)
}

#' Get diverging colors
#'
#' Return a legacy corrplot 0.95 diverging palette. The neutral anchor is white.
#'
#' @param diverging Diverging palette name. One of RdBu, BrBG, PiYG, PRGn,
#'   PuOr, or RdYlBu.
#' @param n Number of colors to return; must be a positive integer.
#' @return A character vector of hexadecimal colors.
#' @seealso [grDevices::colorRampPalette()]
#' @export
COL2 <- function(diverging = c("RdBu", "BrBG", "PiYG", "PRGn", "PuOr", "RdYlBu"),
                 n = 200) {
  diverging <- match.arg(diverging)
  .corrplot_ng_validate_palette_length(n)
  colors <- switch(
    diverging,
    RdBu = c("#67001F", "#B2182B", "#D6604D", "#F4A582", "#FDDBC7", "#FFFFFF",
             "#D1E5F0", "#92C5DE", "#4393C3", "#2166AC", "#053061"),
    BrBG = c("#543005", "#8C510A", "#BF812D", "#DFC27D", "#F6E8C3", "#FFFFFF",
             "#C7EAE5", "#80CDC1", "#35978F", "#01665E", "#003C30"),
    PiYG = c("#8E0152", "#C51B7D", "#DE77AE", "#F1B6DA", "#FDE0EF", "#FFFFFF",
             "#E6F5D0", "#B8E186", "#7FBC41", "#4D9221", "#276419"),
    PRGn = c("#40004B", "#762A83", "#9970AB", "#C2A5CF", "#E7D4E8", "#FFFFFF",
             "#D9F0D3", "#A6DBA0", "#5AAE61", "#1B7837", "#00441B"),
    PuOr = c("#7F3B08", "#B35806", "#E08214", "#FDB863", "#FEE0B6", "#FFFFFF",
             "#D8DAEB", "#B2ABD2", "#8073AC", "#542788", "#2D004B"),
    RdYlBu = c("#A50026", "#D73027", "#F46D43", "#FDAE61", "#FEE090", "#FFFFFF",
               "#E0F3F8", "#ABD9E9", "#74ADD1", "#4575B4", "#313695")
  )
  grDevices::colorRampPalette(colors)(n)
}

.corrplot_ng_validate_palette_length <- function(n) {
  if (!is.numeric(n) || length(n) != 1L || is.na(n) || !is.finite(n) ||
      n < 1 || n != as.integer(n)) {
    stop("n must be a positive integer", call. = FALSE)
  }
  invisible(as.integer(n))
}
