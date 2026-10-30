% Generated for corrplot-ng legacy palette compatibility.
\name{COL1}
\alias{COL1}
\title{Get legacy sequential colors}
\usage{
COL1(sequential = c("Oranges", "Purples", "Reds", "Blues", "Greens",
  "Greys", "OrRd", "YlOrRd", "YlOrBr", "YlGn"), n = 200)
}
\arguments{
\item{sequential}{Sequential palette family.}
\item{n}{Positive integer number of colors to return.}
}
\value{A character vector of hexadecimal colors.}
\description{Returns the corrplot 0.95 sequential palette for the requested family, interpolated to \code{n} colors.}
\seealso{\code{COL2}, \code{grDevices::colorRampPalette}}
